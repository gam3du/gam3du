//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "just a demo"
)]

use gam3du_framework::init_logger;
use gam3du_framework_common::message::ServerToClientMessage;
use std::{cell::RefCell, time::Duration};
use tracing::info;
use wasm_bindgen::prelude::*;
use wasm_rs_shared_channel::spsc::{self, SharedChannel};

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
pub fn run() -> Result<(), JsValue> {
    info!("run");
    APPLICATION_STATE.with_borrow_mut(|state| {
        let Some(receiver) = &mut state.receiver else {
            return Err(JsValue::from_str("cannot run without a receiver"));
        };

        info!("waiting for message");
        loop {
            match receiver.recv(Some(Duration::from_millis(100))) {
                Ok(None) => {
                    info!("… still waiting for message …");
                }
                Ok(Some(response)) => {
                    info!("received message: {response:?}");
                    break Ok(());
                }
                Err(err) => {
                    break Err(JsValue::from(format!(
                        "Error while waiting for message: {err:?}"
                    )));
                }
            }
        }
    })?;

    info!("PythonRuntime run terminated");
    Ok(())
}
