// has false positives; enable every now and then to see whether there are actually missed opportunities
#![allow(missing_copy_implementations)]
// usually too noisy. Disable every now and then to see whether there are actually identifiers that need to be improved.
#![allow(unused_crate_dependencies)]
// TODO re-enable this later and review all occurrences
#![allow(clippy::cast_precision_loss)]
// TODO remove before release
#![allow(clippy::missing_panics_doc)]
#![allow(missing_docs)]
#![allow(clippy::print_stdout)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::indexing_slicing)]
#![allow(clippy::panic)]

// TODO enable hand-picked clippy lints from the `restriction` group

use std::{sync::mpsc::channel, thread};

use gam3du::framework;
use gam3du::logging::init_logger;
use gam3du::python::runner;

fn main() {
    //ecs_test();

    init_logger();

    let (command_sender, command_receiver) = channel();

    let source_path = "python/test.py";
    let python_tread = thread::spawn(move || runner(&source_path, command_sender));

    pollster::block_on(framework::start("demo scene".into(), command_receiver));
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    python_tread.join().unwrap();
}
