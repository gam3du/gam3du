//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "just a demo"
)]

use gam3du_framework::init_logger;
use gam3du_framework_common::{
    api::ApiDescriptor, api_channel::WasmApiClientEndpoint, message::ServerToClientMessage,
};
use runtime_python::PythonRuntimeBuilder;
use std::{cell::RefCell, path::Path};
use tracing::{debug, info};
use wasm_bindgen::prelude::*;
use wasm_rs_shared_channel::spsc::{self, SharedChannel};

const API_JSON: &str = include_str!("../../../../applications/robot/control.api.json");

#[wasm_bindgen(raw_module = "./worker.mjs")]
extern "C" {
    /// sends requests to an api server (the game engine)
    fn send_api_client_request(client_to_server_request: &[u8]);
}

struct ApplicationState {
    receiver: Option<spsc::Receiver<ServerToClientMessage>>,
}

impl ApplicationState {
    const fn new() -> Self {
        Self { receiver: None }
    }
}

thread_local! {
    static APPLICATION_STATE: RefCell<ApplicationState> = const { RefCell::new(ApplicationState::new()) };
}

#[wasm_bindgen]
pub fn init() {
    init_logger();
    info!("initialized");
}

#[wasm_bindgen]
pub fn set_channel_buffers(buffers: JsValue) {
    info!("set_channel_buffers");

    // assert_eq!(array.length(), 2);

    let channel = SharedChannel::from(buffers);
    let (_sender, receiver) = channel.split();

    APPLICATION_STATE.with_borrow_mut(|state| {
        assert!(
            state.receiver.replace(receiver).is_none(),
            "receiver has already been set"
        );
    });
    info!("channel buffers successfully set");
}

#[wasm_bindgen]
pub fn run(source: &str) -> Result<(), JsValue> {
    info!("run");

    // let source = include_str!("../../../../applications/robot/python/control/robot.py");

    info!("creating python runtime builder for engine plugin");
    let mut python_runtime_builder = PythonRuntimeBuilder::new(
        Path::new("../../../../applications/robot/python/control"),
        "robot",
    );

    let robot_control_module = rustpython::vm::py_freeze!(
        module_name = "robot",
        file = "../../../applications/robot/python/control/robot.py"
    )
    .decode();

    let robot_control_api_module = rustpython::vm::py_freeze!(
        module_name = "robot_control_api",
        file = "../../../applications/robot/python/control/robot_control_api.py"
    )
    .decode();

    let robot_control_api_async_module = rustpython::vm::py_freeze!(
        module_name = "robot_control_api_async",
        file = "../../../applications/robot/python/control/robot_control_api_async.py"
    )
    .decode();

    let robot_api: ApiDescriptor = serde_json::from_str(API_JSON).map_err(|err| err.to_string())?;

    APPLICATION_STATE.with_borrow_mut(|state| {
        let Some(receiver) = state.receiver.take() else {
            return Err(JsValue::from_str("cannot run without a receiver"));
        };

        let send = |request: &[u8]| {
            debug!("sender callback with {} bytes", request.len());
            send_api_client_request(request);
        };

        let api_client = WasmApiClientEndpoint::new(robot_api, receiver, Box::from(send));

        python_runtime_builder.add_api_client(Box::from(api_client));
        python_runtime_builder.add_frozen_module("robot", robot_control_module);
        python_runtime_builder.add_frozen_module("robot_api", robot_control_api_module);
        python_runtime_builder.add_frozen_module("robot_api_async", robot_control_api_async_module);

        let mut runtime = python_runtime_builder.build();

        info!("starting Python runtime for control script");
        // runtime.enter_main();
        runtime
            .run_source(source)
            .map_err(|err| format!("{err:#?}"))?;

        Ok(())
    })?;

    info!("PythonRuntime run terminated");
    Ok(())
}
