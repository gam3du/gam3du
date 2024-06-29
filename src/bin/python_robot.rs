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

use gam3du::api::{Api, Identifier};
use gam3du::logging::init_logger;
use gam3du::python::runner;
use gam3du::{framework, Command};
use tiny_http::{Response, Server};

fn main() {
    //ecs_test();

    init_logger();

    let api_json = std::fs::read_to_string("apis/robot.api.json").unwrap();
    let api: Api = serde_json::from_str(&api_json).unwrap();

    let (command_sender, command_receiver) = channel();

    let python_thread = {
        let source_path = "python/test.py";
        let command_sender = command_sender.clone();
        let api = api.clone();
        thread::spawn(move || runner(&source_path, command_sender, &api))
    };

    let webserver_tread = {
        let command_sender = command_sender.clone();
        let api = api.clone();
        thread::spawn(move || http_server(&command_sender, &api))
    };

    pollster::block_on(framework::start("demo scene".into(), command_receiver));
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    python_thread.join().unwrap();
    webserver_tread.join().unwrap();
}

fn http_server(command_sender: &Sender<Command>, api: &Api) {
    let server = Server::http("0.0.0.0:8000").unwrap();

    for request in server.incoming_requests() {
        let url = request.url();
        let Some(url) = url.strip_prefix(&format!("/{}/", api.name)) else {
            request
                .respond(Response::from_string("unknown api").with_status_code(404))
                .unwrap();
            continue;
        };

        let command = Command {
            name: Identifier(url.to_owned()),
        };

        let response = Response::from_string(format!("{command:?}"));

        command_sender
            .send(Command {
                name: Identifier(url.to_owned()),
            })
            .unwrap();

        request.respond(response).unwrap();
    }
}
