//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(
    clippy::unwrap_used,
    clippy::unwrap_in_result,
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "TODO remove before release"
)]

mod api_client;

// this `use`-clause is required to suppress the warning about unused crates
// this dependency is required in order to configure the `wasm_js` feature
use getrandom as _;

use api_client::WasmApiClientEndpoint;
use gam3du_framework::init_logger;
use gam3du_framework_common::{api::ApiDescriptor, message::ServerToClientMessage};
use runtime_python::PythonRuntimeBuilder;
use std::{cell::RefCell, path::Path};
use tracing::info;
use wasm_bindgen::prelude::*;
use wasm_rs_shared_channel::spsc::{self, SharedChannel};

const API_JSON: &str = include_str!("../../../control.api.json");

// #[wasm_bindgen(raw_module = "./worker.mjs")]
// extern "C" {
//     /// sends requests to an api server (the game engine)
//     fn send_api_client_request(client_to_server_request: &[u8]);
// }

struct ApplicationState {
    receiver: Option<spsc::Receiver<ServerToClientMessage>>,
    // sender: Option<MessagePort>,
}

impl ApplicationState {
    const fn new() -> Self {
        Self {
            receiver: None,
            // sender: None,
        }
    }
}

thread_local! {
    static APPLICATION_STATE: RefCell<ApplicationState> = const { RefCell::new(ApplicationState::new()) };
}

// TODO make this a main method
#[wasm_bindgen]
pub fn init() {
    init_logger();
    info!("initialized");
}

// TODO rename this to `init`
#[wasm_bindgen]
pub fn set_channel_buffers(buffers: JsValue) {
    info!("init");

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

    info!("creating python runtime builder for engine plugin");
    let mut python_runtime_builder = PythonRuntimeBuilder::new(
        Path::new("../../../../applications/robot/python/control"),
        "robot",
    );

    let robot_control_module = rustpython::vm::py_freeze!(
        module_name = "robot",
        file = "../../python/control/robot.py"
    )
    .decode();

    let robot_control_api_module = rustpython::vm::py_freeze!(
        module_name = "robot_control_api",
        file = "../../python/control/robot_control_api.py"
    )
    .decode();

    let robot_control_api_async_module = rustpython::vm::py_freeze!(
        module_name = "robot_control_api_async",
        file = "../../python/control/robot_control_api_async.py"
    )
    .decode();

    let robot_api: ApiDescriptor = serde_json::from_str(API_JSON).map_err(|err| err.to_string())?;

    APPLICATION_STATE.with_borrow_mut(|state| {
        let Some(receiver) = state.receiver.take() else {
            return Err(JsValue::from_str("cannot run without a receiver"));
        };

        let api_client = WasmApiClientEndpoint::new(robot_api, receiver);

        python_runtime_builder.add_api_client(Box::from(api_client));
        python_runtime_builder.add_frozen_module("robot", robot_control_module);
        python_runtime_builder.add_frozen_module("robot_api", robot_control_api_module);
        python_runtime_builder.add_frozen_module("robot_api_async", robot_control_api_async_module);

        let mut runtime = python_runtime_builder.build();

        info!("starting Python runtime for control script");
        runtime
            .run_source(source)
            .map_err(|err| format!("{err:#?}"))?;

        Ok(())
    })?;

    info!("PythonRuntime run terminated");
    Ok(())
}
