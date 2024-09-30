#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "TODO remove before release"
)]

use engine_robot::{plugin::PythonPlugin, GameLoop, RendererBuilder};
use gam3du_framework::{application::Application, logging::init_logger};
use gam3du_framework_common::{
    api::{self, ApiDescriptor},
    event::{ApplicationEvent, EngineEvent},
};
use log::{debug, error};
use runtime_python::{PythonRunnerThread, PythonRuntimeBuilder};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{channel, Sender},
    },
    thread,
};
use winit::event_loop::{ControlFlow, EventLoop};

static EXIT_FLAG: AtomicBool = AtomicBool::new(false);

fn main() {
    init_logger();

    let (event_sender, event_receiver) = channel();

    // notify the main loop if CTRL+C was pressed
    register_ctrlc(&event_sender);

    // let webserver_thread = {
    //     let event_sender = event_sender.clone();
    //     let api = api.clone();
    //     thread::spawn(move || {
    //         debug!("thread[webserver]: starting server");
    //         http_server(&event_sender, &api, &EXIT_FLAG);
    //         debug!("thread[webserver]: exit");
    //     })
    // };

    let (python_thread, robot_api_engine_endpoint) = start_python_robot(
        "launchers/robot/control.api.json",
        "launchers/robot/python/control",
        "robot",
    );

    let window_event_loop = EventLoop::with_user_event().build().unwrap();
    window_event_loop.set_control_flow(ControlFlow::Poll);
    let window_proxy = window_event_loop.create_proxy();

    let (game_state_sender, game_state_receiver) = channel();

    let game_loop_thread = {
        thread::spawn(move || {
            // the game loop might not be `Send`, so we need to create it from withon the thread
            let mut game_loop = GameLoop::default();
            let mut python_runtime_builder =
                PythonRuntimeBuilder::new("launchers/robot/python/plugin", "robot_plugin");
            python_runtime_builder.add_api_server(robot_api_engine_endpoint);
            let plugin = PythonPlugin::new(python_runtime_builder);

            // let mut plugin = NativePlugin::new();
            // plugin.add_robot_controller(robot_api_engine_endpoint);
            game_loop.add_plugin(plugin);

            // the game state is needed in the main window's loop so we send a reference thereof out of this thread
            game_state_sender.send(game_loop.clone_state()).unwrap();

            debug!("thread[game loop]: starting game loop");
            game_loop.run(&event_receiver);
            debug!("thread[game loop]: game loop returned");

            // shut down everything

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

    // wait for the game loop to send us a copy of its state, so that we can pass it to the renderer
    let game_state = game_state_receiver.recv().unwrap();

    let mut application = pollster::block_on(Application::new(
        "Robot".into(),
        event_sender.clone(),
        RendererBuilder::new(game_state),
    ));

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
    if let Err(error) = python_thread.join() {
        error!("python thread joined: {error:?}");
    }

    // debug!("Waiting for webserver to exit …");
    // webserver_thread.join().unwrap();
}

fn start_python_robot(
    robot_api_descriptor_path: &str,
    python_sys_path: &str,
    python_main_module: &str,
) -> (PythonRunnerThread, api::ApiServerEndpoint) {
    let api_json = std::fs::read_to_string(robot_api_descriptor_path).unwrap();
    let robot_api: ApiDescriptor = serde_json::from_str(&api_json).unwrap();

    let (robot_api_script_endpoint, robot_api_engine_endpoint) = api::channel(robot_api);

    let mut python_builder = PythonRuntimeBuilder::new(python_sys_path, python_main_module);

    let user_signal_sender = python_builder.enable_user_signals();
    python_builder.add_api_client(robot_api_script_endpoint);
    let python_runner_thread = python_builder.build_runner_thread(user_signal_sender);

    (python_runner_thread, robot_api_engine_endpoint)
}

fn register_ctrlc(event_sender: &Sender<EngineEvent>) {
    ctrlc::set_handler({
        let event_sender = event_sender.clone();
        move || {
            debug!("CTRL + C received");
            drop(event_sender.send(ApplicationEvent::Exit.into()));
        }
    })
    .expect("Error setting Ctrl-C handler");
}

// fn http_server(command_sender: &Sender<EngineEvent>, api: &Api, exit_flag: &'static AtomicBool) {
//     let server = Server::http("0.0.0.0:8000").unwrap();

//     'next_request: loop {
//         let request = match server.recv_timeout(Duration::from_millis(50)) {
//             Ok(Some(request)) => request,
//             Ok(None) => {
//                 if exit_flag.load(Ordering::Relaxed) {
//                     break 'next_request;
//                 }
//                 continue 'next_request;
//             }
//             Err(error) => {
//                 error!("{error}");
//                 break 'next_request;
//             }
//         };

//         let url = request.url();
//         let Some(url) = url.strip_prefix(&format!("/{}/", api.name)) else {
//             request
//                 .respond(Response::from_string("unknown api").with_status_code(404))
//                 .unwrap();
//             continue;
//         };

//         let command = Identifier(url.to_owned());

//         let response = Response::from_string(format!("{command:?}"));

//         // FIXME: Extract parameters from response
//         command_sender
//             .send(EngineEvent::RobotEvent {
//                 command: Identifier(url.to_owned()),
//                 parameters: vec![],
//             })
//             .unwrap();

//         request.respond(response).unwrap();
//     }
// }
