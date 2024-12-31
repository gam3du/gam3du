#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    // clippy::expect_used,
    // clippy::todo,
    // clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    // clippy::unwrap_in_result,
    clippy::panic_in_result_fn,

    reason = "TODO remove before release"
)]

mod api_endpoint;

use crate::api_endpoint::WasmApiServerEndpoint;
use application_robot::WINDOW_TITLE;
use engine_robot::{plugin::PythonPlugin, GameLoop, GameState, RendererBuilder};
use gam3du_framework::application::{Application, GameLoopRunner};
use gam3du_framework_common::{
    api::ApiDescriptor, event::FrameworkEvent, message::ServerToClientMessage,
};
use runtime_python::PythonRuntimeBuilder;
use std::{
    cell::RefCell,
    collections::VecDeque,
    mem,
    path::Path,
    rc::Rc,
    sync::{mpsc, Arc, RwLock},
};
use tracing::{debug, info, trace};
use wasm_bindgen::prelude::*;
use wasm_bindgen::{
    prelude::{wasm_bindgen, Closure},
    JsCast,
};
use wasm_rs_shared_channel::spsc;
use web_sys::{
    js_sys::{self, Uint8Array},
    MessageChannel, MessageEvent, MessagePort, Worker, WorkerOptions, WorkerType,
};
use web_time::Instant;
use winit::{event_loop::EventLoop, platform::web::EventLoopExtWeb};

// const CONTROL_API_PATH: &str = "applications/robot/control.api.json";
const API_JSON: &str = include_str!("../../../control.api.json");

const CHANNEL_CAPACITY: u32 = 0x1_0000;

pub(crate) struct ApplicationState {
    server_port: Option<MessagePort>,
    client_port: Option<MessagePort>,
    pub(crate) client_messages: VecDeque<Vec<u8>>,
}

impl ApplicationState {
    const INIT: Self = Self {
        server_port: None,
        client_port: None,
        client_messages: VecDeque::new(),
    };
}

thread_local! {
    pub(crate) static APPLICATION_STATE: RefCell<ApplicationState> = const { RefCell::new(ApplicationState::INIT) };
}

#[wasm_bindgen(start)]
fn main() -> Result<(), wasm_bindgen::JsValue> {
    gam3du_framework::init_logger();
    tracing::info!("RobotWebMain loaded");
    Ok(())
}

// fn create_storage() -> StaticStorage {
//     let mut storage = StaticStorage::default();

//     storage.store(
//         Path::new(CONTROL_API_PATH),
//         include_bytes!("../control.api.json").into(),
//     );

//     storage
// }

#[wasm_bindgen]
pub fn init() -> Result<(), JsValue> {
    info!("initializing application");

    info!("creating message channel");
    let command_channel = MessageChannel::new().unwrap();
    let server_port = command_channel.port1();
    let client_port = command_channel.port2();

    info!("registering event handler for incoming client messages");

    let client_message_handler = get_on_client_message();
    server_port.set_onmessage(Some(client_message_handler.as_ref().unchecked_ref()));
    mem::forget(client_message_handler);

    APPLICATION_STATE.with_borrow_mut(|state| {
        info!("storing server port in application state");
        assert!(
            state.server_port.replace(server_port).is_none(),
            "server port has already been set"
        );
        info!("storing client port in application state");
        assert!(
            state.client_port.replace(client_port).is_none(),
            "client port has already been set"
        );
    });

    info!("application successfully initialized");
    Ok(())
}

/// Create a closure to act on the messages sent by the client
fn get_on_client_message() -> Closure<dyn FnMut(MessageEvent)> {
    Closure::new(move |event: MessageEvent| {
        let data = event.data();
        debug!("Received request: {data:?}");

        let bytes = Uint8Array::new(&data).to_vec();

        APPLICATION_STATE.with_borrow_mut(|state| {
            debug!("enqueueing client message: {bytes:?}");
            state.client_messages.push_back(bytes);
        });

        // let result = match event.data().as_bool().unwrap() {
        //     true => "even",
        //     false => "odd",
        // };

        // let document = web_sys::window().unwrap().document().unwrap();
        // document
        //     .get_element_by_id("resultField")
        //     .expect("#resultField should exist")
        //     .dyn_ref::<HtmlElement>()
        //     .expect("#resultField should be a HtmlInputElement")
        //     .set_inner_text(result);
    })
}

// /// connect an api client to this instance by sending its underlying share buffers
// #[wasm_bindgen]
// pub fn connect_api_client(message_port: MessagePort) {
//     info!("connect_api_client");

//     // let channel = spsc::SharedChannel::from(channel_buffers);
//     // let (sender, _receiver) = channel.split();

//     APPLICATION_STATE.with_borrow_mut(|state| {
//         assert!(
//             state.server_port.replace(message_port).is_none(),
//             "sender has already been set"
//         );
//     });
//     info!("sender successfully set");
// }

struct Runner {
    timestamp: Instant,
    game_loop: GameLoop<PythonPlugin>,
    event_source: mpsc::Receiver<FrameworkEvent>,
}

impl Runner {
    fn new(
        game_loop: GameLoop<PythonPlugin>,
        event_receiver: mpsc::Receiver<FrameworkEvent>,
    ) -> Self {
        Self {
            timestamp: Instant::now(),
            game_loop,
            event_source: event_receiver,
        }
    }
}

impl GameLoopRunner for Runner {
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

static GAME_STATE: Option<Arc<RwLock<Box<GameState>>>> = None;

#[wasm_bindgen]
pub fn reset() -> Result<(), JsValue> {
    if let Some(state) = &GAME_STATE {
        *state.write().unwrap() = Box::new(GameState::new((10, 10)));
    }

    Ok(())
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    info!("creating framework event channel");
    let (event_sender, event_receiver) = mpsc::channel();

    info!("creating framework event channel");
    // this is used to send framework messages to the event loop
    // at the moment only `Exit` will be handled and no one ever issues this event
    // remember to trigger a window proxy to make sure this event will be polled
    let (_window_event_sender, framework_event_receiver) = mpsc::channel();

    // info!("creating static file storage");
    // let storage = create_storage();

    info!("creating python runtime builder for engine plugin");
    let mut python_runtime_builder = PythonRuntimeBuilder::new(
        Path::new("applications/robot/python/plugin"),
        "robot_plugin",
    );

    let robot_plugin_module = rustpython::vm::py_freeze!(
        module_name = "robot_plugin",
        // Some how this path makes the Rust-Analyzer emit an error while the compiler accepts it
        file = "../../python/plugin/robot_plugin.py"
    )
    .decode();

    python_runtime_builder.add_frozen_module("robot_plugin", robot_plugin_module);

    // info!("creating channel for a control script to communicate with the plugin");
    // let (robot_control_api_client_endpoint, robot_control_api_engine_endpoint) = start_python_robot(
    //     Path::new("applications/robot/python/control"),
    //     "robot",
    // );

    // let api_json = storage.get_content(Path::new(CONTROL_API_PATH)).unwrap();
    let robot_api: ApiDescriptor = serde_json::from_str(API_JSON).unwrap();

    let api_server_message_port = APPLICATION_STATE
        .with_borrow_mut(|state| state.server_port.take())
        .ok_or("api client sender not initialized")?;
    let robot_control_api_engine_endpoint = WasmApiServerEndpoint::new(
        robot_api,
        api_server_message_port,
        // Box::new(poll_api_client_request),
    );

    // let mut python_builder = PythonRuntimeBuilder::new(python_sys_path, python_main_module);

    // let user_signal_sender = python_builder.enable_user_signals();
    // python_builder.add_api_client(robot_control_api_client_endpoint);
    // let python_runner_thread = python_builder.build_runner_thread();

    info!("connecting control channel to plugin");
    python_runtime_builder.add_api_server(robot_control_api_engine_endpoint);

    info!("creating engine plugin");
    let plugin = PythonPlugin::new(python_runtime_builder);

    info!("creating initial game state");
    let game_state = GameState::new((10, 10));
    let shared_game_state = game_state.into_shared();
    let mut game_loop = GameLoop::new(Arc::clone(&shared_game_state));
    game_loop.add_plugin(plugin);

    info!("creating game loop runner");
    let game_loop_runner = Runner::new(game_loop, event_receiver);

    info!("creating renderer builder");
    let renderer_builder = RendererBuilder::new(
        shared_game_state,
        include_str!("../../../shaders/robot.wgsl").into(),
    );

    info!("creating application");
    let application = Application::new(
        WINDOW_TITLE,
        event_sender,
        renderer_builder,
        framework_event_receiver,
        game_loop_runner,
    );

    info!("creating event loop");
    let event_loop = EventLoop::new().map_err(|err| err.to_string())?;
    // event_loop.set_control_flow(ControlFlow::Poll); // this causes too much load in the browser
    // let event_loop_proxy = event_loop.create_proxy(); // currently unused

    info!("starting application event loop...");
    event_loop.spawn_app(application);

    info!("application started successfully");
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkerState {
    Loading,
    Initializing,
    Ready,
    Running,
}

#[wasm_bindgen]
pub struct PythonWorker {
    worker_state: Rc<RefCell<WorkerState>>,
    worker: Rc<RefCell<Worker>>,
}

#[wasm_bindgen]
impl PythonWorker {
    #[wasm_bindgen(constructor)]
    #[allow(
        clippy::new_without_default,
        reason = "this is meant to be used as a JavaScript constructor"
    )]
    pub fn new() -> Self {
        info!("creating new Python runtime");

        let command_port = APPLICATION_STATE
            .with_borrow(|state| state.client_port.clone())
            .unwrap();

        info!("creating buffers backing the message channel");
        let (sender, receiver) = spsc::channel::<ServerToClientMessage>(CHANNEL_CAPACITY).split();

        debug!("command port: {command_port:?}");

        debug!("setting onmessage handler for server messages on command port");
        let server_message_handler = Self::get_on_server_message(sender);
        command_port.set_onmessage(Some(server_message_handler.as_ref().unchecked_ref()));
        mem::forget(server_message_handler);

        let worker_state = Rc::new(RefCell::new(WorkerState::Loading));

        debug!("creating new WebWorker");
        let worker_options = WorkerOptions::new();
        worker_options.set_type(WorkerType::Module);
        let worker =
            Worker::new_with_options("./runtime-python/worker.mjs", &worker_options).unwrap();
        let worker = Rc::new(RefCell::new(worker));
        debug!("registering handler for incoming worker messages");
        let worker_message_handler = Self::get_on_worker_message(
            receiver,
            command_port,
            Rc::clone(&worker_state),
            Rc::clone(&worker),
        );
        worker
            .borrow_mut()
            .set_onmessage(Some(worker_message_handler.as_ref().unchecked_ref()));
        mem::forget(worker_message_handler);

        debug!("PythonWorker successfully created");
        Self {
            worker_state,
            worker,
        }
    }

    /// Create a closure to act on the messages sent by the server.
    ///
    /// called whenever the engine sends a response message back to the python runtime
    /// the message will be forwarded to the shared channel of the blocked worker
    fn get_on_server_message(
        sender: spsc::Sender<ServerToClientMessage>,
    ) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::new(move |message_event: MessageEvent| {
            let data = message_event.data();
            debug!("on_server_message: {data:?}");

            let bytes = Uint8Array::new(&data).to_vec();
            let message = bincode::deserialize(&bytes).unwrap();

            sender.send(&message).unwrap();
        })
    }

    // #[wasm_bindgen]
    // pub async fn init(&mut self) {
    //     info!("starting Python Worker");
    //     let worker = Worker::new("./runtime-python/worker.mjs"); //, { type: "module" });
    //                                                              // self.worker.onmessage = self.on_worker_loaded;

    //     info!("Waiting for PythonWorker to be loaded");
    //     self.send_async(null, "loading").await;
    //     info!("PythonWorker has been loaded successfully");

    //     // let message = {
    //     //     type: "set_channel_buffers",
    //     //     buffers: self.shared_channel.buffers(),
    //     // };
    //     info!("sending channel buffers to PythonWorker", message);
    //     self.send_async(message, "setting channel buffers").await;

    //     assert!(
    //         self.worker.replace(worker).is_none(),
    //         "worker has already been created"
    //     );

    //     info!("PythonWorker has been configured", message);
    // }

    // /// Sends a message to the worker and waits for a response.
    // ///
    // /// This is a convenience function to simplify the initial setup and will not work if the runtime
    // /// is blocked.
    // async fn send_async(payload: &[u8], topic: &str) {
    //     return Promise::new(|resolve, reject| {
    //         let worker = self.worker;

    //         fn unexpected(message_event: MessageEvent) {
    //             let message = message_event.data;
    //             error!(
    //                 "unexpected message from worker while no handler was registered:",
    //                 message
    //             );
    //         }

    //         // message event handler forwarding all PythonRequests to the Application
    //         fn on_worker_response(message_event: MessageEvent) {
    //             let message = message_event.data;
    //             trace!("response from worker", message);
    //             resolve(message);
    //             worker.onmessage = unexpected;
    //         }

    //         worker.onmessage = on_worker_response;

    //         if (payload != null) {
    //             worker.postMessage(payload);
    //         }
    //     });
    // }

    // message event handler forwarding all PythonRequests to the Application

    fn get_on_worker_message(
        receiver: spsc::Receiver<ServerToClientMessage>,
        command_port: MessagePort,
        worker_state: Rc<RefCell<WorkerState>>,
        worker: Rc<RefCell<Worker>>,
    ) -> Closure<dyn FnMut(MessageEvent)> {
        Closure::new(move |message_event: MessageEvent| {
            let message = message_event.data();
            let mut worker_state = worker_state.borrow_mut();
            trace!("message from worker (state: {worker_state:?}): {message:?}");

            match *worker_state {
                WorkerState::Loading => {
                    let receiver = receiver.0.clone().into();

                    let message = js_sys::Object::new();
                    assert!(
                        js_sys::Reflect::set(&message, &"type".into(), &"init".into()).unwrap(),
                        "failed to set `type` field of message"
                    );
                    assert!(
                        js_sys::Reflect::set(&message, &"receiver".into(), &receiver).unwrap(),
                        "failed to set `receiver` field of message"
                    );

                    info!("sending shared channel to PythonWorker: {message:?}");
                    worker.borrow().post_message(&message).unwrap();

                    info!("Loading: loading acknowledged by Worker; switch mode to `Initializing`");
                    *worker_state = WorkerState::Initializing;
                }
                WorkerState::Initializing => {
                    info!("Initializing: initialization acknowledged by Worker; switch mode to `Ready`");
                    *worker_state = WorkerState::Ready;
                }
                WorkerState::Ready => {
                    // panic!("No messages are expected in ready state");
                    // switch to `running` state so that all further messages from the Worker will be blindly
                    // forwarded to the engine
                    info!("Ready: run command acknowledged by Worker; switch mode to `Running`");
                    *worker_state = WorkerState::Running;
                }
                WorkerState::Running => {
                    info!("Running: forwarding worker command to engine server");
                    command_port.post_message(&message).unwrap();
                }
            }
        })
    }

    // fn on_worker_message(&mut self, message_event: MessageEvent) {

    // }

    pub fn run(&mut self, source: &str) {
        info!("starting PythonRuntime (this will block the containing WebWorker until the script completes)");

        let worker_state = self.worker_state.borrow_mut();
        assert_eq!(
            *worker_state,
            WorkerState::Ready,
            "Worker needs to be ready to run"
        );

        let message = js_sys::Object::new();
        assert!(
            js_sys::Reflect::set(&message, &"type".into(), &"run".into()).unwrap(),
            "failed to set `type` field of message"
        );
        assert!(
            js_sys::Reflect::set(&message, &"source".into(), &source.into()).unwrap(),
            "failed to set `source` field of message"
        );

        info!("sending run-command to PythonModule WASM: {message:?}");
        self.worker.borrow().post_message(&message).unwrap();
    }

    #[wasm_bindgen]
    pub fn kill(self) {
        self.worker.borrow().terminate();
    }
}
