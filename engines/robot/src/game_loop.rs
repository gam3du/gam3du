use crate::{plugin, SharedGameState};
use gam3du_framework_common::event::{ApplicationEvent, FrameworkEvent};
use log::{debug, warn};
use std::{
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc,
    },
    thread,
};
use web_time::{Duration, Instant};

/// Number of game loop iterations per second.
/// This is a multiple of common frame rates.
const TICKS_PER_SECOND: u32 = 240;

/// Duration of each game tick. Same as
/// `Duration::from_secs_f64(f64::from(TICKS_PER_SECOND).recip())`
/// but with const support
const TICK_DURATION: Duration = Duration::from_nanos(
    (1_000_000_000_u64 + TICKS_PER_SECOND as u64 / 2) / TICKS_PER_SECOND as u64,
);

/// Time to wait before the game loop stops trying to catch up to real time
/// giving giving the current thread a chance do its work
const TIMEOUT: Duration = Duration::from_millis(1000);

/// The root object of a running engine
pub struct GameLoop<Plugin: plugin::Plugin> {
    /// Contains the current state which will be updated by the game loop.
    /// This might be shared with renderers.
    /// In order to allow multiple renderers, this is a `RwLock` rather than a `Mutex`.
    game_state: SharedGameState,
    plugin: Option<Plugin>,
}

impl<Plugin: plugin::Plugin> GameLoop<Plugin> {
    #[must_use]
    pub fn new(game_state: SharedGameState) -> Self {
        Self {
            game_state,
            plugin: None,
        }
    }

    pub fn run(mut self, event_source: &Receiver<FrameworkEvent>) {
        self.init();

        let mut due = Instant::now();
        while let Some(due_next) = self.progress(event_source, due) {
            due = due_next;
        }
    }

    pub fn init(&mut self) {
        if let Some(plugin) = &mut self.plugin {
            let mut game_state = self.game_state.write().unwrap();

            plugin.init(&mut game_state);
        }
    }

    pub fn progress(
        &mut self,
        event_source: &Receiver<FrameworkEvent>,
        mut tick_time: Instant,
    ) -> Option<Instant> {
        // // wait until we've reached the target time,
        // // giving the renderer some time to fetch the current state
        // if let Some(delay) = tick_time.checked_duration_since(Instant::now()) {
        // FIXME PLEASE!
        //     thread::sleep(delay);
        // }

        // lock game_state for the entire scope
        let mut game_state = self.game_state.write().unwrap();
        // timeout to make sure the lock and current thread aren't blocked for too long
        let timeout = Instant::now();
        // statistics for overload handling
        let mut too_slow_count = 0;
        'next_tick: loop {
            // drain event queue
            'next_event: loop {
                match event_source.try_recv() {
                    Ok(engine_event) => match engine_event {
                        FrameworkEvent::Window { event } => {
                            debug!("{event:?}");
                        }
                        FrameworkEvent::Device { event } => {
                            debug!("{event:?}");
                        }
                        FrameworkEvent::Application { event } => match event {
                            ApplicationEvent::Exit => {
                                debug!("Received Exit-event. Exiting game loop");
                                return None;
                            }
                        },
                    },
                    Err(TryRecvError::Disconnected) => {
                        debug!("Event source disconnected. Exiting game loop");
                        return None;
                    }
                    Err(TryRecvError::Empty) => break 'next_event,
                }
            }

            // run scripting runtimes here
            if let Some(plugin) = &mut self.plugin {
                plugin.update(&mut game_state);
            }

            // perform the actual state-transition for this tick
            game_state.update();

            // compute the timestamp of the next game loop iteration
            tick_time += TICK_DURATION;
            // see whether the next tick is due
            if Instant::now() < tick_time {
                // there's still some time left; yielding to caller
                break 'next_tick;
            }

            // game loop is running too slow so we don't give back our lock, yet
            too_slow_count += 1;

            // prevent endless-looping
            if timeout.elapsed() >= TIMEOUT {
                warn!("Game loop wasn't able to keep up for at least {TIMEOUT:?} ({too_slow_count} ticks) - yielding to caller");
                break 'next_tick;
            }
        }

        // see you then!
        Some(tick_time)
    }

    #[must_use]
    pub fn clone_state(&self) -> SharedGameState {
        Arc::clone(&self.game_state)
    }

    pub fn add_plugin(&mut self, plugin: Plugin) {
        assert!(
            self.plugin.replace(plugin).is_none(),
            "only one plugin can be set for now"
        );
    }
}
