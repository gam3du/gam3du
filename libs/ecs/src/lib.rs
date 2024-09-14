#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::missing_panics_doc,
    clippy::print_stdout,
    clippy::unwrap_used,
    // clippy::expect_used,
    // clippy::indexing_slicing,
    // clippy::panic,
    clippy::todo,
    reason = "TODO remove before release"
)]

pub mod component;
pub mod entity;
pub mod event_subscriber;
mod runtime;
pub mod state;
pub mod transform;

use std::sync::{Arc, RwLock};

#[derive(Default)]
pub struct Application {
    pub state: Arc<RwLock<state::State>>,

    application_runtime: runtime::ApplicationRuntime,
}

impl Application {
    pub fn start(&mut self) {
        let runtime = &mut self.application_runtime;

        runtime.start(&self.state);
    }

    #[must_use]
    pub fn get_state_arc(&self) -> Arc<RwLock<state::State>> {
        Arc::clone(&self.state)
    }
}
