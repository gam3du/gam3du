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

use std::sync::mpsc::channel;
use std::sync::{mpsc::Sender, Arc};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use bindings::event::{ApplicationEvent, EngineEvent};
use bindings::{
    api::{Api, Identifier},
    event::EventRouter,
};
use engine_robot::GameLoop;
use gam3du_framework::framework::Application;
use gam3du_framework::logging::init_logger;
use log::info;
use tiny_http::{Response, Server};
use winit::event_loop::{ControlFlow, EventLoop};

fn main() {
    init_logger();

    let game_loop = GameLoop::default();
    let game_state = Arc::clone(&game_loop.game_state);

    let api_json = std::fs::read_to_string("engines/robot/api.json").unwrap();
    let api: Api = serde_json::from_str(&api_json).unwrap();

    let mut event_router = EventRouter::default();
    let event_sender = event_router.clone_sender();

    let game_loop_thread = {
        let (sender, receiver) = channel();
        event_router.add_handler(Box::new(move |event| match event {
            api_call @ EngineEvent::ApiCall { .. } => {
                sender.send(api_call).unwrap();
                None
            }
            EngineEvent::Application {
                event: ApplicationEvent::Exit,
            } => {
                sender.send(event.clone()).unwrap();
                Some(event)
            }
            other => Some(other),
        }));
        thread::spawn(move || game_loop.run(&receiver))
    };

    let python_thread = {
        let source_path = "python/test.py";
        let event_sender = event_sender.clone();
        let api = api.clone();
        thread::spawn(move || bind_python::runner(&source_path, event_sender, &api))
    };

    let webserver_thread = {
        let event_sender = event_sender.clone();
        let api = api.clone();
        thread::spawn(move || http_server(&event_sender, &api))
    };

    let mut application = pollster::block_on(Application::new(
        "demo scene".into(),
        &mut event_router,
        game_state,
    ));

    // framework::start(application);
    let event_loop = EventLoop::with_user_event().build().unwrap();

    // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
    // dispatched any events. This is ideal for games and similar applications.
    event_loop.set_control_flow(ControlFlow::Poll);

    let proxy = event_loop.create_proxy();

    event_router.add_handler(Box::new(move |engine_event| match engine_event {
        event @ EngineEvent::Application {
            event: ApplicationEvent::Exit,
        } => {
            proxy.send_event(event.clone()).unwrap();
            Some(event)
        }
        other => Some(other),
    }));

    let event_thread = thread::spawn(move || event_router.run());

    //let app = Application::new(title, receiver, event_sender);
    log::info!("Entering event loop...");
    event_loop.run_app(&mut application).unwrap();

    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    // FIXME Event thread doesn't exit, yet
    // python_thread.join().unwrap();
    // game_loop_thread.join().unwrap();
    // webserver_tread.join().unwrap();
    // event_thread.join().unwrap();

    let mut python_thread = Some(python_thread);
    let mut game_loop_thread = Some(game_loop_thread);
    let mut webserver_thread = Some(webserver_thread);
    let mut event_thread = Some(event_thread);
    while python_thread.is_some()
        || game_loop_thread.is_some()
        || webserver_thread.is_some()
        || event_thread.is_some()
    {
        info!("Waiting for all threads to exit …");
        if python_thread.as_ref().is_some_and(JoinHandle::is_finished) {
            info!("Python stopped");
            python_thread.take().unwrap().join().unwrap();
        }
        if game_loop_thread
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            info!("Game loop stopped");
            game_loop_thread.take().unwrap().join().unwrap();
        }
        if webserver_thread
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            info!("webserver stopped");
            webserver_thread.take().unwrap().join().unwrap();
        }
        if event_thread.as_ref().is_some_and(JoinHandle::is_finished) {
            info!("event router stopped");
            event_thread.take().unwrap().join().unwrap();
        }

        thread::sleep(Duration::from_secs(1));
    }
}

fn http_server(command_sender: &Sender<EngineEvent>, api: &Api) {
    let server = Server::http("0.0.0.0:8000").unwrap();

    for request in server.incoming_requests() {
        let url = request.url();
        let Some(url) = url.strip_prefix(&format!("/{}/", api.name)) else {
            request
                .respond(Response::from_string("unknown api").with_status_code(404))
                .unwrap();
            continue;
        };

        let command = Identifier(url.to_owned());

        let response = Response::from_string(format!("{command:?}"));

        command_sender
            .send(EngineEvent::ApiCall {
                api: Identifier("robot".into()),
                command: Identifier(url.to_owned()),
            })
            .unwrap();

        request.respond(response).unwrap();
    }
}
