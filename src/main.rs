#![warn(clippy::all, clippy::pedantic)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]

// TODO enable hand-picked clippy lints from the `restriction` group

//mod application_state;
mod framework;
mod logging;
mod python;
mod scene;

use std::{
    sync::{atomic::AtomicU16, mpsc::channel},
    thread,
};

use logging::init_logger;
use python::python_runner;

pub(crate) static ROTATION: AtomicU16 = AtomicU16::new(0);

fn main() {
    init_logger();

    let (command_sender, command_receiver) = channel();

    let source_path = "python/test.py";
    let python_tread = thread::spawn(move || python_runner(&source_path, command_sender));

    pollster::block_on(framework::start("demo scene".into(), command_receiver));
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    python_tread.join().unwrap();
}
