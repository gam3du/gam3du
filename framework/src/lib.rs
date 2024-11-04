#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::cast_precision_loss,
    clippy::expect_used,
    // clippy::indexing_slicing,
    // clippy::missing_errors_doc,
    // clippy::missing_panics_doc,
    // clippy::panic,
    // clippy::print_stdout,
    clippy::todo,
    clippy::unwrap_used,
    reason = "TODO remove before release"
)]

use gam3du_framework_common::event::{ApplicationEvent, FrameworkEvent};
use log::debug;
use std::sync::mpsc::Sender;

pub mod application;
pub mod logging;
mod render_surface;
pub mod renderer;

/// notify a connected receiver if CTRL+C was pressed
///
/// # Panics
/// Will return an error if a system error occurred while setting the handler.
pub fn register_ctrlc(event_sender: &Sender<FrameworkEvent>) {
    ctrlc::set_handler({
        let event_sender = event_sender.clone();
        move || {
            debug!("CTRL+C received");
            drop(event_sender.send(ApplicationEvent::Exit.into()));
        }
    })
    .expect("Error setting Ctrl+C handler");
}
