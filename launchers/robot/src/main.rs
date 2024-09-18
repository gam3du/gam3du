#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    // clippy::missing_panics_doc,
    // clippy::print_stdout,
    clippy::unwrap_used,
    clippy::expect_used,
    // clippy::indexing_slicing,
    // clippy::panic,
    reason = "TODO remove before release"
)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::time::Duration;
use std::{sync::mpsc::channel, thread};

use engine_robot::{GameLoop, RendererBuilder};
use gam3du_framework::application::Application;
use gam3du_framework::logging::init_logger;
use log::{debug, error};
use runtimes::api::{Api, Identifier};
use runtimes::event::{ApplicationEvent, EngineEvent};
use tiny_http::{Response, Server};
use winit::event_loop::{ControlFlow, EventLoop};

static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

fn main() {
    init_logger();

    let game_loop = GameLoop::default();
    let (event_sender, event_receiver) = channel();

    let api_json = std::fs::read_to_string("engines/robot/api.json").unwrap();
    let api: Api = serde_json::from_str(&api_json).unwrap();

    ctrlc::set_handler({
        let event_sender = event_sender.clone();
        move || {
            debug!("CTRL + C received");
            drop(event_sender.send(ApplicationEvent::Exit.into()));
        }
    })
    .expect("Error setting Ctrl-C handler");

    let python_thread = runtime_python::run("python".into(), "robot".into(), event_sender.clone());

    let webserver_thread = {
        let event_sender = event_sender.clone();
        let api = api.clone();
        thread::spawn(move || {
            debug!("thread[webserver]: starting server");
            http_server(&event_sender, &api, &EXIT_FLAG);
            debug!("thread[webserver]: exit");
        })
    };

    let mut application = pollster::block_on(Application::new(
        "Robot".into(),
        event_sender.clone(),
        RendererBuilder::new(game_loop.clone_state()),
    ));

    let window_event_loop = EventLoop::with_user_event().build().unwrap();
    window_event_loop.set_control_flow(ControlFlow::Poll);
    let window_proxy = window_event_loop.create_proxy();

    let game_loop_thread = {
        thread::spawn(move || {
            debug!("thread[game loop]: starting game loop");
            game_loop.run(&event_receiver);
            debug!("thread[game loop]: game loop returned");
            debug!("thread[game loop]: instruct window event loop to stop now");
            window_proxy
                .send_event(ApplicationEvent::Exit.into())
                .unwrap();
            debug!("thread[game loop]: instruct python vm to stop now");
            python_thread.stop();
            debug!("thread[game loop]: instruct webserver to stop now");
            EXIT_FLAG.store(true, Ordering::Relaxed);
            debug!("thread[game loop]: exit");
            python_thread
        })
    };

    log::info!("main: Entering event loop...");
    window_event_loop.run_app(&mut application).unwrap();
    drop(application);
    log::debug!("main: window event loop exited");
    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    // Every thread should have received an exit-notification by now

    debug!("Waiting for game loop to exit …");
    #[expect(
        clippy::shadow_unrelated,
        reason = "this is related, but got moved through a foreign thread"
    )]
    let python_thread = game_loop_thread.join().unwrap();

    debug!("Waiting for python vm to exit …");
    python_thread.join().unwrap();

    debug!("Waiting for webserver to exit …");
    webserver_thread.join().unwrap();
}

fn http_server(command_sender: &Sender<EngineEvent>, api: &Api, exit_flag: &'static AtomicBool) {
    let server = Server::http("0.0.0.0:8000").unwrap();

    'next_request: loop {
        let request = match server.recv_timeout(Duration::from_millis(50)) {
            Ok(Some(request)) => request,
            Ok(None) => {
                if exit_flag.load(Ordering::Relaxed) {
                    break 'next_request;
                }
                continue 'next_request;
            }
            Err(error) => {
                error!("{error}");
                break 'next_request;
            }
        };

        let url = request.url();
        let Some(url) = url.strip_prefix(&format!("/{}/", api.name)) else {
            request
                .respond(Response::from_string("unknown api").with_status_code(404))
                .unwrap();
            continue;
        };

        let command = Identifier(url.to_owned());

        let response = Response::from_string(format!("{command:?}"));

        // FIXME: Extract parameters from response
        command_sender
            .send(EngineEvent::RobotEvent {
                command: Identifier(url.to_owned()),
                parameters: vec![],
            })
            .unwrap();

        request.respond(response).unwrap();
    }
}
