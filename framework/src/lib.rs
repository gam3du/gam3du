#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::cast_precision_loss,
    clippy::expect_used,
    clippy::indexing_slicing,
    // clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    // clippy::panic,
    // clippy::print_stdout,
    clippy::todo,
    clippy::unwrap_used,
    reason = "TODO remove before release"
)]

pub mod api;
pub mod application;
pub mod event;
mod graphics_context;
pub mod logging;
pub mod message;
pub mod module;
mod surface_wrapper;
