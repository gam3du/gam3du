#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    // clippy::expect_used,
    reason = "TODO remove before release"
)]

use engine_robot::{plugin::PythonPlugin, GameLoop, GameState, RendererBuilder, SharedGameState};
use gam3du_framework::{application::Application, logging::init_logger};
use gam3du_framework_common::{
    api::{self, ApiDescriptor, ApiServerEndpoint},
    event::{ApplicationEvent, FrameworkEvent},
};
use log::{debug, error, info, trace, warn};
use runtime_python::PythonRuntimeBuilder;
use rustpython_vm::signal::{UserSignal, UserSignalSender};
use std::{
    fmt::{self, Display},
    fs::read_to_string,
    process::ExitCode,
    sync::{self, mpsc::channel, Arc},
    thread,
    time::{Duration, Instant},
};
use tokio::join;
use tokio_util::sync::CancellationToken;
use winit::event_loop::{ControlFlow, EventLoop, EventLoopProxy};

const WINDOW_TITLE: &str = "Robot";

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

    let local_set = tokio::task::LocalSet::new();

    let runtime = tokio::runtime::Builder::new_current_thread()
        .on_thread_park(|| trace!("tokio::on_thread_park"))
        .on_thread_start(|| trace!("tokio::on_thread_start"))
        .on_thread_stop(|| trace!("tokio::on_thread_stop"))
        .on_thread_unpark(|| trace!("tokio::on_thread_unpark"))
        .enable_time()
        .build()
        .map_err(ApplicationError::BuildRuntime)?;

    local_set.block_on(&runtime, async_main())
    // runtime.block_on(async_main())
}

#[allow(clippy::too_many_lines, reason = "TODO split this up later")]
async fn async_main() -> ApplicationResult<()> {
    let cancellation_token = CancellationToken::new();

    let ctrl_c_task = ctrl_c_task(cancellation_token.clone());

    let (event_sender, event_receiver) = channel();
    let (window_event_sender, window_event_receiver) = channel();
    // register_ctrlc(&event_sender);

    let game_state = GameState::new((10, 10));
    let shared_game_state = game_state.into_shared();

    let (main_window_task, window_proxy) = open_main_window(
        Arc::clone(&shared_game_state),
        event_sender.clone(),
        window_event_receiver,
    );

    let (python_thread, python_signal_handler, robot_api_engine_endpoint) = start_python_robot(
        "applications/robot/control.api.json",
        "applications/robot/python/control",
        "robot",
    );

    let game_loop_thread = start_game_loop(
        Arc::clone(&shared_game_state),
        robot_api_engine_endpoint,
        event_receiver,
    );

    // FIXME on Windows the window will still be unresponsively lingering until the control was given back to the OS (maybe a bug in `winit`)
    // main_window_task.await.unwrap();

    info!("Normal operation. Waiting for any task to terminate …");
    let mut debug_timer = Instant::now();
    loop {
        if ctrl_c_task.is_finished() {
            info!("CTRL+C task completed first");
            break;
        }
        if main_window_task.is_finished() {
            info!("main window task completed first");
            break;
        }
        if python_thread.is_finished() {
            info!("python thread completed first");
            break;
        }
        if game_loop_thread.is_finished() {
            info!("game loop thread completed first");
            break;
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
        if debug_timer.elapsed() > Duration::from_secs(1) {
            debug!("waiting for any task to terminate …");
            debug_timer = Instant::now();
        }
    }

    if cancellation_token.is_cancelled() {
        debug!("cancellation via token is already in progress");
    } else {
        debug!("cancelling all tasks via token");
        cancellation_token.cancel();
    }

    match window_event_sender.send(ApplicationEvent::Exit.into()) {
        Ok(()) => {
            window_proxy.wake_up();
            debug!("exit event successfully sent to main window");
        }
        Err(error) => {
            debug!("failed to send exit event to main window: {error}");
        }
    }

    {
        debug!("cancelling python runtime via interrupt");
        // todo move this back into some helper (see PythonRunnerThread::stop)
        let make_interrupt: UserSignal = Box::new(|vm| {
            // Copied from rustpython_vm::stdlib::signal::_signal::default_int_handler
            let exec_type = vm.ctx.exceptions.keyboard_interrupt.to_owned();
            Err(vm.new_exception_empty(exec_type))
        });
        match python_signal_handler.send(make_interrupt) {
            Ok(()) => {
                debug!("interrupt successfully sent to python runtime");
            }
            Err(error) => {
                error!("failed to send interrupt to python runtime: {error}");
            }
        }
    }

    {
        match event_sender.send(FrameworkEvent::Application {
            event: ApplicationEvent::Exit,
        }) {
            Ok(()) => {
                debug!("exit event successfully sent to game loop");
            }
            Err(error) => {
                error!("failed to send exit event to game loop: {error}");
            }
        }
    }

    info!("waiting for all remaining tasks to complete …");
    let (ctrl_c_task_result, main_window_task_result) = join!(ctrl_c_task, main_window_task);
    debug!("all tasks completed");

    match main_window_task_result {
        Ok(Ok(())) => {
            info!("main window task terminated normally");
        }
        Ok(Err(application_error)) => {
            error!("main window task terminated with application error: {application_error}");
        }
        Err(join_error) => {
            error!("main window task terminated with join error: {join_error}");
        }
    }

    match ctrl_c_task_result {
        Ok(()) => {
            info!("CTRL+C task terminated normally");
        }
        Err(join_error) => {
            error!("CTRL+C task terminated with join error: {join_error}");
        }
    }

    debug!("Waiting for game loop to exit …");
    match game_loop_thread.join() {
        Ok(()) => {
            info!("game loop thread terminated normally");
        }
        Err(error) => {
            error!("game loop thread terminated with error: {error:?}");
        }
    }

    debug!("Waiting for python vm to exit …");
    match python_thread.join() {
        Ok(()) => {
            info!("python thread terminated normally");
        }
        Err(error) => {
            error!("python thread terminated with error: {error:?}");
        }
    }

    Ok(())
}

fn ctrl_c_task(cancellation_token: CancellationToken) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn_local(async move {
        tokio::select! {
            result = tokio::signal::ctrl_c() => {
                match result {
                    Ok(()) => {
                        info!("CTRL+C pressed");
                    },
                    Err(error) => {
                        warn!("cannot wait for CTRL+C: {error}");
                        // wait for regular external cancellation instead
                        cancellation_token.cancelled().await;
                    }
                }
            }
            () = cancellation_token.cancelled() => {
                debug!("CTRL+C-task has been cancelled by token");
            }
        }
    })
}

fn open_main_window(
    shared_game_state: SharedGameState,
    event_sender: sync::mpsc::Sender<FrameworkEvent>,
    framework_events: sync::mpsc::Receiver<FrameworkEvent>,
) -> (
    tokio::task::JoinHandle<ApplicationResult<()>>,
    EventLoopProxy,
) {
    let window_event_loop = EventLoop::new().unwrap();
    window_event_loop.set_control_flow(ControlFlow::Poll);
    let window_proxy = window_event_loop.create_proxy();

    let main_window_task = tokio::task::spawn_local(async move {
        let mut application = Application::new(
            WINDOW_TITLE,
            event_sender,
            RendererBuilder::new(shared_game_state),
            framework_events,
        )
        .await;

        log::info!("main: Entering event loop...");
        window_event_loop.run_app(&mut application).unwrap();
        drop(application);
        log::debug!("main: window event loop exited");

        Ok(())
    });

    (main_window_task, window_proxy)
}

fn start_game_loop(
    game_state: SharedGameState,
    robot_api_engine_endpoint: ApiServerEndpoint,
    event_receiver: sync::mpsc::Receiver<FrameworkEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut python_runtime_builder =
            PythonRuntimeBuilder::new("applications/robot/python/plugin", "robot_plugin");
        python_runtime_builder.add_api_server(robot_api_engine_endpoint);
        let plugin = PythonPlugin::new(python_runtime_builder);

        // the game loop might not be `Send`, so we need to create it from within the thread
        let mut game_loop = GameLoop::new(game_state);

        // let mut plugin = NativePlugin::new();
        // plugin.add_robot_controller(robot_api_engine_endpoint);
        game_loop.add_plugin(plugin);

        debug!("thread[game loop]: starting game loop");
        game_loop.run(&event_receiver);
        debug!("thread[game loop]: game loop returned");
    })
}

fn start_python_robot(
    robot_api_descriptor_path: &str,
    python_sys_path: &str,
    python_main_module: &str,
) -> (thread::JoinHandle<()>, UserSignalSender, ApiServerEndpoint) {
    let api_json = read_to_string(robot_api_descriptor_path).unwrap();
    let robot_api: ApiDescriptor = serde_json::from_str(&api_json).unwrap();

    let (robot_api_script_endpoint, robot_api_engine_endpoint) = api::channel(robot_api);

    let mut python_builder = PythonRuntimeBuilder::new(python_sys_path, python_main_module);

    let user_signal_sender = python_builder.enable_user_signals();
    python_builder.add_api_client(robot_api_script_endpoint);
    let python_runner_thread = python_builder.build_runner_thread();

    (
        python_runner_thread,
        user_signal_sender,
        robot_api_engine_endpoint,
    )
}

type ApplicationResult<T> = Result<T, ApplicationError>;

#[derive(Debug)]
enum ApplicationError {
    #[allow(
        unused,
        reason = "This is a placeholder for errors that still need to be categorized"
    )]
    Todo(String),
    BuildRuntime(std::io::Error),
}

impl Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApplicationError::Todo(message) => write!(formatter, "other error: {message}"),
            ApplicationError::BuildRuntime(error) => {
                write!(formatter, "failed to build async runtime: {error}")
            }
        }
    }
}

impl From<ApplicationError> for ExitCode {
    fn from(value: ApplicationError) -> Self {
        match value {
            ApplicationError::Todo(_) => ExitCode::FAILURE,
            ApplicationError::BuildRuntime(_) => ExitCode::from(2),
        }
    }
}
