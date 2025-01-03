//! Native implementation of the Robot game

#![expect(
    clippy::todo,
    clippy::unwrap_used,
    reason = "TODO remove before launch"
)]

use application_robot::APPLICATION_TITLE;
use engine_robot::{plugin::PythonPlugin, GameLoop, GameState, RendererBuilder};
use gam3du_framework::{
    application::{Application, GameLoopRunner},
    init_logger,
};
use gam3du_framework_common::module::Module;
use gam3du_framework_common::{
    api::ApiDescriptor,
    api_channel::{NativeApiClientEndpoint, NativeApiServerEndpoint},
    event::{ApplicationEvent, FrameworkEvent},
};
use lib_file_storage::{FileStorage, StaticStorage};
use runtime_python::PythonRuntimeBuilder;
use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{self, mpsc::channel, Arc},
    thread::JoinHandle,
};
use tracing::{debug, error, info};
use web_time::Instant;
use winit::event_loop::{ControlFlow, EventLoop};

fn main() -> ExitCode {
    match guarded_main() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            error!("application exited with error: {err}");
            err.into()
        }
    }
}

fn guarded_main() -> ApplicationResult<()> {
    init_logger();

    // let local_set = tokio::task::LocalSet::new();

    // let runtime = tokio::runtime::Builder::new_current_thread()
    //     .on_thread_park(|| trace!("tokio::on_thread_park"))
    //     .on_thread_start(|| trace!("tokio::on_thread_start"))
    //     .on_thread_stop(|| trace!("tokio::on_thread_stop"))
    //     .on_thread_unpark(|| trace!("tokio::on_thread_unpark"))
    //     .enable_time()
    //     .build()
    //     .map_err(ApplicationError::BuildRuntime)?;

    async_main()

    // local_set.block_on(&runtime, async_main())
    // runtime.block_on(async_main())
}

struct NativeGameLoopRunner {
    timestamp: Instant,
    game_loop: GameLoop<PythonPlugin>,
    event_source: sync::mpsc::Receiver<FrameworkEvent>,
}

impl NativeGameLoopRunner {
    fn new(
        robot_api_engine_endpoint: NativeApiServerEndpoint,
        game_state: Arc<sync::RwLock<Box<GameState>>>,
        event_receiver: sync::mpsc::Receiver<FrameworkEvent>,
    ) -> Self {
        let mut python_runtime_builder = PythonRuntimeBuilder::new(
            Path::new("applications/robot/python/plugin"),
            "robot_plugin",
        );

        let robot_plugin_module = rustpython::vm::py_freeze!(
            module_name = "robot_plugin",
            // Some how this path makes the Rust-Analyzer emit an error while the compiler accepts it
            file = "../python/plugin/robot_plugin.py"
        )
        .decode();

        python_runtime_builder.add_frozen_module("robot_plugin", robot_plugin_module);

        info!("connecting control channel to plugin");
        python_runtime_builder.add_api_server(robot_api_engine_endpoint);

        info!("creating engine plugin");
        let plugin = PythonPlugin::new(python_runtime_builder);

        // the game loop might not be `Send`, so we need to create it from within the thread
        let mut game_loop = GameLoop::new(game_state);

        // let mut plugin = NativePlugin::new();
        // plugin.add_robot_controller(robot_api_engine_endpoint);
        game_loop.add_plugin(plugin);

        // debug!("thread[game loop]: starting game loop");
        // game_loop.run(&event_receiver);
        // debug!("thread[game loop]: game loop returned");

        game_loop.init();
        Self {
            timestamp: Instant::now(),
            game_loop,
            event_source: event_receiver,
        }
    }
}

impl GameLoopRunner for NativeGameLoopRunner {
    fn init(&mut self) {
        self.timestamp = Instant::now();
        self.game_loop.init();
    }

    fn update(&mut self) {
        if let Some(timestamp) = self.game_loop.progress(&self.event_source, self.timestamp) {
            self.timestamp = timestamp;
        } else {
            todo!("do not crash on exit");
        }
    }
}

// #[expect(clippy::too_many_lines, reason = "TODO split this up later")]
#[expect(clippy::unnecessary_wraps, reason = "TODO")]
fn async_main() -> ApplicationResult<()> {
    let mut storage = StaticStorage::default();

    storage.store(
        Path::new("applications/robot/control.api.json"),
        include_bytes!("../../control.api.json").into(),
    );

    let window_event_loop = EventLoop::new().unwrap();
    window_event_loop.set_control_flow(ControlFlow::Poll);
    let window_proxy = window_event_loop.create_proxy();

    // let cancellation_token = CancellationToken::new();

    // let ctrl_c_task = ctrl_c_task(cancellation_token.clone());

    let (event_sender, event_receiver) = channel();
    let (window_event_sender, window_event_receiver) = channel();
    // register_ctrlc(&event_sender);

    let game_state = GameState::new((10, 10));
    let shared_game_state = game_state.into_shared();

    // let (main_window_task, window_proxy) = open_main_window(
    //     Arc::clone(&shared_game_state),
    //     event_sender.clone(),
    //     window_event_receiver,
    // );

    // let (python_thread, python_signal_handler, robot_api_engine_endpoint) = start_python_robot(
    let (_python_thread, engine_server_endpoint) = start_python_robot(
        &storage,
        Path::new("applications/robot/control.api.json"),
        Path::new("../applications/robot/python/control").to_path_buf(),
        "robot",
    );

    // let (robot_api_script_endpoint, robot_api_engine_endpoint) = api::channel(robot_api);

    // let game_loop_thread = {
    //     let game_state = Arc::clone(&shared_game_state);
    //     thread::spawn(move || {})
    // };

    let game_loop_runner = NativeGameLoopRunner::new(
        engine_server_endpoint,
        Arc::clone(&shared_game_state),
        event_receiver,
    );

    // let runner = GameLoopRunner::new(robot_api_engine_endpoint, game_state, event_receiver);

    let mut application = Application::new(
        APPLICATION_TITLE,
        event_sender,
        RendererBuilder::new(
            shared_game_state,
            include_str!("../../shaders/robot.wgsl").into(),
        ),
        window_event_receiver,
        // move || {
        game_loop_runner, // },
    );

    info!("main: Entering event loop...");
    window_event_loop.run_app(&mut application).unwrap(); // blocking!
    drop(application);
    debug!("main: window event loop exited");

    // info!("Normal operation. Waiting for any task to terminate …");
    // let mut debug_timer = Instant::now();
    // loop {
    //     // if ctrl_c_task.is_finished() {
    //     //     info!("CTRL+C task completed first");
    //     //     break;
    //     // }
    //     // if main_window_task.is_finished() {
    //     //     info!("main window task completed first");
    //     //     break;
    //     // }
    //     // if python_thread.is_finished() {
    //     //     info!("python thread completed first");
    //     //     break;
    //     // }
    //     if game_loop_thread.is_finished() {
    //         info!("game loop thread completed first");
    //         break;
    //     }

    //     thread::sleep(Duration::from_millis(50));
    //     // tokio::time::sleep(Duration::from_millis(50)).await;
    //     if debug_timer.elapsed() > Duration::from_secs(1) {
    //         debug!("waiting for any task to terminate …");
    //         debug_timer = Instant::now();
    //     }
    // }

    // if cancellation_token.is_cancelled() {
    //     debug!("cancellation via token is already in progress");
    // } else {
    //     debug!("cancelling all tasks via token");
    //     cancellation_token.cancel();
    // }

    match window_event_sender.send(ApplicationEvent::Exit.into()) {
        Ok(()) => {
            window_proxy.wake_up();
            debug!("exit event successfully sent to main window");
        }
        Err(error) => {
            debug!("failed to send exit event to main window: {error}");
        }
    }

    // {
    //     debug!("cancelling python runtime via interrupt");
    //     // todo move this back into some helper (see PythonRunnerThread::stop)
    //     let make_interrupt: UserSignal = Box::new(|vm| {
    //         // Copied from rustpython_vm::stdlib::signal::_signal::default_int_handler
    //         let exec_type = vm.ctx.exceptions.keyboard_interrupt.to_owned();
    //         Err(vm.new_exception_empty(exec_type))
    //     });
    //     match python_signal_handler.send(make_interrupt) {
    //         Ok(()) => {
    //             debug!("interrupt successfully sent to python runtime");
    //         }
    //         Err(error) => {
    //             error!("failed to send interrupt to python runtime: {error}");
    //         }
    //     }
    // }

    // {
    //     match event_sender.send(FrameworkEvent::Application {
    //         event: ApplicationEvent::Exit,
    //     }) {
    //         Ok(()) => {
    //             debug!("exit event successfully sent to game loop");
    //         }
    //         Err(error) => {
    //             error!("failed to send exit event to game loop: {error}");
    //         }
    //     }
    // }

    // info!("waiting for all remaining tasks to complete …");
    // let (ctrl_c_task_result, main_window_task_result) = join!(ctrl_c_task, main_window_task);
    // debug!("all tasks completed");

    // match main_window_task_result {
    //     Ok(Ok(())) => {
    //         info!("main window task terminated normally");
    //     }
    //     Ok(Err(application_error)) => {
    //         error!("main window task terminated with application error: {application_error}");
    //     }
    //     Err(join_error) => {
    //         error!("main window task terminated with join error: {join_error}");
    //     }
    // }

    // match ctrl_c_task_result {
    //     Ok(()) => {
    //         info!("CTRL+C task terminated normally");
    //     }
    //     Err(join_error) => {
    //         error!("CTRL+C task terminated with join error: {join_error}");
    //     }
    // }

    // debug!("Waiting for game loop to exit …");
    // match game_loop_thread.join() {
    //     Ok(()) => {
    //         info!("game loop thread terminated normally");
    //     }
    //     Err(error) => {
    //         error!("game loop thread terminated with error: {error:?}");
    //     }
    // }

    // debug!("Waiting for python vm to exit …");
    // match python_thread.join() {
    //     Ok(()) => {
    //         info!("python thread terminated normally");
    //     }
    //     Err(error) => {
    //         error!("python thread terminated with error: {error:?}");
    //     }
    // }

    Ok(())
}

// fn ctrl_c_task(cancellation_token: CancellationToken) -> tokio::task::JoinHandle<()> {
//     tokio::task::spawn_local(async move {
//         tokio::select! {
//             // TODO not supported on WASM
//             // result = tokio::signal::ctrl_c() => {
//             //     match result {
//             //         Ok(()) => {
//             //             info!("CTRL+C pressed");
//             //         },
//             //         Err(error) => {
//             //             warn!("cannot wait for CTRL+C: {error}");
//             //             // wait for regular external cancellation instead
//             //             cancellation_token.cancelled().await;
//             //         }
//             //     }
//             // }
//             () = cancellation_token.cancelled() => {
//                 debug!("CTRL+C-task has been cancelled by token");
//             }
//         }
//     })
// }

// fn open_main_window(
//     shared_game_state: SharedGameState,
//     event_sender: sync::mpsc::Sender<FrameworkEvent>,
//     framework_events: sync::mpsc::Receiver<FrameworkEvent>,
// ) -> (
//     // tokio::task::JoinHandle<ApplicationResult<()>>,
//     EventLoopProxy,
// ) {
//     let window_event_loop = EventLoop::new().unwrap();
//     window_event_loop.set_control_flow(ControlFlow::Poll);
//     let window_proxy = window_event_loop.create_proxy();

//     let main_window_task = tokio::task::spawn_local(async move {
//         let mut application = Application::new(
//             WINDOW_TITLE,
//             event_sender,
//             RendererBuilder::new(shared_game_state),
//             framework_events,
//         );

//         log::info!("main: Entering event loop...");
//         window_event_loop.run_app(&mut application).unwrap();
//         drop(application);
//         log::debug!("main: window event loop exited");

//         Ok(())
//     });

//     (main_window_task, window_proxy)
// }

fn start_python_robot(
    storage: &dyn FileStorage,
    robot_api_descriptor_path: &Path,
    python_sys_path: PathBuf,
    python_main_module: impl Into<String> + Send + 'static,
) -> (
    JoinHandle<()>,
    // UserSignalSender,
    NativeApiServerEndpoint,
) {
    let api_json = storage
        .get_content(Path::new(robot_api_descriptor_path))
        .unwrap();
    let api: ApiDescriptor = serde_json::from_slice(&api_json).unwrap();

    let (script_to_engine_sender, script_to_engine_receiver) = channel();
    let (engine_to_script_sender, engine_to_script_receiver) = channel();

    let server_endpoint = NativeApiServerEndpoint::new(
        api.clone(),
        script_to_engine_receiver,
        engine_to_script_sender,
    );

    let client_endpoint =
        NativeApiClientEndpoint::new(api, script_to_engine_sender, engine_to_script_receiver);

    let python_runner_thread = std::thread::Builder::new()
        // .stack_size(10 * 1024 * 1024)
        .spawn(move || {
            let mut python_builder =
                PythonRuntimeBuilder::new(&python_sys_path, python_main_module);

            // let user_signal_sender = python_builder.enable_user_signals();
            python_builder.add_api_client(Box::from(client_endpoint));
            debug!("thread[python]: start interpreter");
            let mut runtime = python_builder.build();
            runtime.enter_main();
        })
        .unwrap();

    (
        python_runner_thread,
        //  user_signal_sender,
        server_endpoint,
    )
}

type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug)]
enum ApplicationError {
    // #[allow(
    //     unused,
    //     reason = "This is a placeholder for errors that still need to be categorized"
    // )]
    // Todo(String),
    // BuildRuntime(std::io::Error),
}

impl Display for ApplicationError {
    fn fmt(&self, _formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        // match self {
        // ApplicationError::Todo(message) => write!(formatter, "other error: {message}"),
        // ApplicationError::BuildRuntime(error) => {
        //     write!(formatter, "failed to build async runtime: {error}")
        // }
        // }
        Ok(())
    }
}

impl From<ApplicationError> for ExitCode {
    fn from(_value: ApplicationError) -> Self {
        todo!();
        // match value {
        //     ApplicationError::Todo(_) => ExitCode::FAILURE,
        //     ApplicationError::BuildRuntime(_) => ExitCode::from(2),
        // }
    }
}
