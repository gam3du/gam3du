use crate::game_state::GameState;
use bindings::event::{ApplicationEvent, EngineEvent};
use log::debug;
use std::{
    sync::{
        mpsc::{Receiver, TryRecvError},
        Arc, RwLock,
    },
    thread,
    time::{Duration, Instant},
};

/// Number of game loop iterations per second.
/// This is a multiple of common frame rates.
const TICKS_PER_SECOND: u32 = 240;

/// Duration of each game tick. Same as
/// `Duration::from_secs_f64(f64::from(TICKS_PER_SECOND).recip())`
/// but with const support
const TICK_DURATION: Duration = Duration::from_nanos(
    (1_000_000_000_u64 + TICKS_PER_SECOND as u64 / 2) / TICKS_PER_SECOND as u64,
);

/// The root object of a running engine
#[derive(Default)]
pub struct GameLoop {
    /// Contains the current state which will be updates by the game loop.
    /// This might be shared with renderers.
    /// In order to allow multiple renderers, this is a `RwLock` rather than a `Mutex`.
    pub game_state: Arc<RwLock<GameState>>,
}

impl GameLoop {
    pub fn run(self, event_source: &Receiver<EngineEvent>) {
        let mut time = Instant::now();
        'game_loop: loop {
            {
                let mut game_state = self.game_state.write().unwrap();
                'next_event: loop {
                    match event_source.try_recv() {
                        Ok(engine_event) => match engine_event {
                            EngineEvent::Window { event: _ } => todo!(),
                            EngineEvent::Device { event: _ } => todo!(),
                            EngineEvent::ApiCall { .. } => todo!(),
                            EngineEvent::RobotEvent { command } => {
                                game_state.process_command(&command);
                            }
                            EngineEvent::Application { event } => match event {
                                ApplicationEvent::Exit => {
                                    debug!("Received Exit-event. Exiting game loop");
                                    break 'game_loop;
                                }
                            },
                        },
                        Err(TryRecvError::Disconnected) => {
                            debug!("Event source disconnected. Exiting game loop");
                            break 'game_loop;
                        }
                        Err(TryRecvError::Empty) => break 'next_event,
                    }
                }

                game_state.update();
            }

            // compute the timestamp of the next game loop iteration
            time += TICK_DURATION;
            if let Some(delay) = time.checked_duration_since(Instant::now()) {
                thread::sleep(delay);
            } else {
                // game loop is running too slow
            }
        }
    }
}
