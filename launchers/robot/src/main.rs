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

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::time::Duration;
use std::{sync::mpsc::channel, thread};

use bind_python::PythonThread;
use bindings::api::{Api, Identifier};
use bindings::event::{ApplicationEvent, EngineEvent};
use engine_robot::{GameLoop, RendererBuilder};
use gam3du_framework::application::Application;
use gam3du_framework::logging::init_logger;
use log::{debug, error, info};
use tiny_http::{Response, Server};
use winit::event_loop::{ControlFlow, EventLoop};

static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

#[allow(clippy::too_many_lines)] // TODO maybe fix later
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

    let python_thread = bind_python::run(event_sender.clone());

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
        event_sender,
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
            python_thread.join().unwrap();
            debug!("thread[game loop]: instruct webserver to stop now");
            EXIT_FLAG.store(true, Ordering::Relaxed);
            debug!("thread[game loop]: exit");
        })
    };

    log::info!("main: Entering event loop...");
    window_event_loop.run_app(&mut application).unwrap();
    drop(application);
    log::debug!("main: window event loop exited");

    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)

    let mut python_thread = None; // Some(python_thread);
    let mut game_loop_thread = Some(game_loop_thread);
    let mut webserver_thread = Some(webserver_thread);
    while python_thread.is_some() || game_loop_thread.is_some() || webserver_thread.is_some() {
        info!("Waiting for all threads to exit …");
        if python_thread
            .as_ref()
            .is_some_and(PythonThread::is_finished)
        {
            info!("Python stopped");
            python_thread.take().unwrap().join().unwrap();
        } else if python_thread.is_some() {
            info!("Waiting for Python");
        }
        if game_loop_thread
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            info!("Game loop stopped");
            game_loop_thread.take().unwrap().join().unwrap();
        } else if game_loop_thread.is_some() {
            info!("Waiting for game loop");
        }
        if webserver_thread
            .as_ref()
            .is_some_and(JoinHandle::is_finished)
        {
            info!("webserver stopped");
            webserver_thread.take().unwrap().join().unwrap();
        } else if webserver_thread.is_some() {
            info!("Waiting for webserver");
        }
        thread::sleep(Duration::from_secs(1));
    }
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
