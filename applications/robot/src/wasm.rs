use crate::error::ApplicationError;
use engine_robot::{plugin::PythonPlugin, GameLoop, GameState, RendererBuilder};
use gam3du_framework::{
    application::{Application, GameLoopRunner},
    init_logger,
};
use gam3du_framework_common::{
    api::{self, ApiClientEndpoint, ApiDescriptor},
    event::FrameworkEvent,
    message::{ClientToServerMessage, ServerToClientMessage},
};
use runtime_python::PythonRuntimeBuilder;
use std::{
    cell::RefCell,
    path::Path,
    sync::{self, mpsc::channel, Arc},
};
use tracing::{info, trace};
use wasm_bindgen::prelude::*;
use wasm_rs_shared_channel::spsc;
use web_time::Instant;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWeb;

const WINDOW_TITLE: &str = "Robot";

// const CONTROL_API_PATH: &str = "applications/robot/control.api.json";
const API_JSON: &str = include_str!("../control.api.json");

#[wasm_bindgen(raw_module = "./module.js")]
extern "C" {
    /// queries for new api requests from a remote controller (e.g. a Python Script)
    fn poll_api_client_request() -> Option<Vec<u8>>;
}

struct ApplicationState {
    api_client_sender: Option<spsc::Sender<ServerToClientMessage>>,
}

impl ApplicationState {
    const INIT: Self = Self {
        api_client_sender: None,
    };
}

thread_local! {
    static APPLICATION_STATE: RefCell<ApplicationState> = const { RefCell::new(ApplicationState::INIT) };
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
    init_logger();
    info!("initialized");

    Ok(())
}

/// connect an api client to this instance by sending its underlying share buffers
#[wasm_bindgen]
pub fn connect_api_client(channel_buffers: JsValue) {
    info!("connect_api_client");

    let channel = spsc::SharedChannel::from(channel_buffers);
    let (sender, _receiver) = channel.split();

    APPLICATION_STATE.with_borrow_mut(|state| {
        assert!(
            state.api_client_sender.replace(sender).is_none(),
            "sender has already been set"
        );
    });
    info!("channel buffers successfully set");
}

struct Runner {
    timestamp: Instant,
    game_loop: GameLoop<PythonPlugin>,
    event_source: sync::mpsc::Receiver<FrameworkEvent>,
    api_client_endpoint: ApiClientEndpoint,
    api_client_sender: spsc::Sender<ServerToClientMessage>,
}

impl Runner {
    fn new(
        game_loop: GameLoop<PythonPlugin>,
        event_receiver: sync::mpsc::Receiver<FrameworkEvent>,
        api_client_endpoint: ApiClientEndpoint,
        api_client_sender: spsc::Sender<ServerToClientMessage>,
    ) -> Self {
        Self {
            timestamp: Instant::now(),
            game_loop,
            event_source: event_receiver,
            api_client_endpoint,
            api_client_sender,
        }
    }
}

impl GameLoopRunner for Runner {
    fn init(&mut self) {
        self.timestamp = Instant::now();
        self.game_loop.init();
    }

    fn update(&mut self) {
        // TODO rather than forwarding all messages between the WASM channels and the Plugin-Endpoints, it would make more sense to implement "WASM-aware" endpoints and send those to the Plugin
        // this requires the Endpoint types to be abstracted as traits and more generics (or dyn) within the Engine/Plugin

        // forward outgoing messages from the plugin to the control script running in a web worker
        while let Some(response) = self.api_client_endpoint.poll_response() {
            trace!("received response from the plugin: {response:#?}");

            trace!("forwarding message to Python Worker");
            self.api_client_sender.send(&response).unwrap();
        }

        // forward incoming messages from the control script running in a web worker to the plugin
        while let Some(request_bytes) = poll_api_client_request() {
            let request = bincode::deserialize(&request_bytes).unwrap();
            trace!("received request from PythonWorker: {request:#?}");

            trace!("forwarding message to plugin");
            let message = ClientToServerMessage::Request(request);
            self.api_client_endpoint.send_to_server(message);
        }

        if let Some(timestamp) = self.game_loop.progress(&self.event_source, self.timestamp) {
            self.timestamp = timestamp;
        } else {
            todo!("do not crash on exit");
        }
    }
}

#[wasm_bindgen]
pub fn start() -> Result<(), JsValue> {
    info!("creating framework event channel");
    let (event_sender, event_receiver) = channel();

    info!("creating framework event channel");
    // this is used to send framework messages to the event loop
    // at the moment only `Exit` will be handled and no one ever issues this event
    // remember to trigger a window proxy to make sure this event will be polled
    let (_window_event_sender, framework_event_receiver) = channel();

    // info!("creating static file storage");
    // let storage = create_storage();

    info!("creating python runtime builder for engine plugin");
    let mut python_runtime_builder = PythonRuntimeBuilder::new(
        Path::new("applications/robot/python/plugin"),
        "robot_plugin",
    );

    info!("creating channel for a control script to communicate with the plugin");
    // let (robot_control_api_client_endpoint, robot_control_api_engine_endpoint) = start_python_robot(
    //     Path::new("applications/robot/python/control"),
    //     "robot",
    // );

    // fn start_python_robot(
    //     _python_sys_path: &Path, = Path::new("applications/robot/python/control"),
    //     _python_main_module: &str, = "robot",
    // ) -> (ApiClientEndpoint, ApiServerEndpoint) {
    // let api_json = storage.get_content(Path::new(CONTROL_API_PATH)).unwrap();
    let robot_api: ApiDescriptor = serde_json::from_str(API_JSON).unwrap();

    let (robot_control_api_client_endpoint, robot_control_api_engine_endpoint) =
        api::channel(robot_api);

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

    let api_client_sender = APPLICATION_STATE
        .with_borrow_mut(|state| state.api_client_sender.take())
        .ok_or("api client sender not initialized")?;

    info!("creating game loop runner");
    let game_loop_runner = Runner::new(
        game_loop,
        event_receiver,
        robot_control_api_client_endpoint,
        api_client_sender,
    );

    info!("creating renderer builder");
    let renderer_builder = RendererBuilder::new(
        shared_game_state,
        include_str!("../../../engines/robot/shaders/robot.wgsl").into(),
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
    let event_loop = EventLoop::new().map_err(ApplicationError::from)?;
    // event_loop.set_control_flow(ControlFlow::Poll); // this causes too much load in the browser
    // let event_loop_proxy = event_loop.create_proxy(); // currently unused

    info!("starting application event loop...");
    event_loop.spawn_app(application);

    info!("application started successfully");
    Ok(())
}
