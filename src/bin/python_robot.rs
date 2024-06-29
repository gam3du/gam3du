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

use std::sync::mpsc::Sender;
use std::{sync::mpsc::channel, thread};

use gam3du::logging::init_logger;
use gam3du::python::runner;
use gam3du::{framework, Command};
use tiny_http::{Response, Server};

fn main() {
    //ecs_test();

    init_logger();

    let (command_sender, command_receiver) = channel();

    let source_path = "python/test.py";
    let python_sender = command_sender.clone();
    let python_tread = thread::spawn(move || runner(&source_path, python_sender));
    let webserver_sender = command_sender.clone();
    let webserver_tread = thread::spawn(move || http_server(&webserver_sender));

    pollster::block_on(framework::start("demo scene".into(), command_receiver));
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    python_tread.join().unwrap();
    webserver_tread.join().unwrap();
}

fn http_server(command_sender: &Sender<Command>) {
    let server = Server::http("0.0.0.0:8000").unwrap();

    for request in server.incoming_requests() {
        match request.url() {
            "/TurnLeft" => {
                command_sender.send(Command::TurnLeft).unwrap();
                request.respond(Response::empty(200)).unwrap();
            }
            "/TurnRight" => {
                command_sender.send(Command::TurnRight).unwrap();
                request.respond(Response::empty(200)).unwrap();
            }
            "/MoveForward" => {
                command_sender.send(Command::MoveForward).unwrap();
                request.respond(Response::empty(200)).unwrap();
            }
            _ => {
                request.respond(Response::empty(404)).unwrap();
            }
        }
    }
}
