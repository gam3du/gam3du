pub mod state;
pub mod entity;
pub mod component;
pub mod event_subscriber;
mod runtime;

use std::{mem, thread, time::Instant};
use std::sync::{Arc, Mutex, RwLock};

pub struct Application {
    pub state: Arc<RwLock<state::State>>,

    application_runtime: runtime::ApplicationRuntime,
}

impl Application {
    pub fn new() -> Application {
        Application {
            state: Arc::new(RwLock::new(state::State::new())),
            application_runtime: runtime::ApplicationRuntime::new(),
        }
    }

    pub fn start(&mut self) {
        let runtime = &mut self.application_runtime;
        
        runtime.start(Arc::clone(&self.state));
    }

    pub fn get_state_arc(&self) -> Arc<RwLock<state::State>> {
        Arc::clone(&self.state)
    }
}