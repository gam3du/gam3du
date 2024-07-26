use std::{
    sync::mpsc::{Receiver, TryRecvError},
    thread,
    time::Duration,
};

use bindings::event::{ApplicationEvent, EngineEvent};

use crate::game_state::GameState;

/// The root object of a running engine
pub struct GameLoop {
    /// Amount of time each iteration of the game loop takes in real time.
    /// This if the reciprocal value of _ticks per _second_.
    tick_duration: Duration,

    game_state: GameState,
}

impl GameLoop {
    fn run(mut self, event_source: &Receiver<EngineEvent>) {
        'game_loop: loop {
            'next_event: loop {
                match event_source.try_recv() {
                    Ok(engine_event) => match engine_event {
                        EngineEvent::Window { event: _ } => todo!(),
                        EngineEvent::Device { event: _ } => todo!(),
                        EngineEvent::ApiCall { api: _, command } => {
                            self.game_state.process_command(&command);
                        }
                        EngineEvent::Application { event } => match event {
                            ApplicationEvent::Exit => break 'game_loop,
                        },
                    },
                    Err(TryRecvError::Disconnected) => break 'game_loop,
                    Err(TryRecvError::Empty) => break 'next_event,
                }
            }

            self.game_state.update();

            // TODO add updater for renderer

            // FIXME this is too imprecise
            thread::sleep(self.tick_duration);
        }
    }
}
