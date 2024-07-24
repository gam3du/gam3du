// has false positives; enable every now and then to see whether there are actually missed opportunities
#![allow(missing_copy_implementations)]
// usually too noisy. Disable every now and then to see whether there are actually identifiers that need to be improved.
#![allow(unused_crate_dependencies)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]
// TODO remove before release
#![allow(clippy::allow_attributes_without_reason)]
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]

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
