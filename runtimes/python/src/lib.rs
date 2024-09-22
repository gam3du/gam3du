#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::todo,
    clippy::panic_in_result_fn,
    reason = "TODO remove and fix before release"
)]

mod api_client;
pub mod bindgen;
mod runner;

pub use runner::{PythonRunnerThread, PythonRuntime, PythonRuntimeBuilder};
