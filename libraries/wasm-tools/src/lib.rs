//! TODO
#![allow(missing_docs, reason = "TODO")]
#![expect(
    clippy::missing_panics_doc,
    clippy::missing_errors_doc,
    reason = "TODO"
)]

use gam3du_framework::init_logger;
use gam3du_framework_common::message::{ErrorResponseMessage, RequestId, ServerToClientMessage};
use std::{cell::RefCell, num::NonZeroU128};
use tracing::info;
use wasm_bindgen::prelude::*;
use wasm_rs_shared_channel::spsc::{self, SharedChannel};

struct ApplicationState {
    sender: Option<spsc::Sender<ServerToClientMessage>>,
}

impl ApplicationState {
    const fn new() -> Self {
        Self { sender: None }
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
    let (sender, _receiver) = channel.split();

    APPLICATION_STATE.with_borrow_mut(|state| {
        assert!(
            state.sender.replace(sender).is_none(),
            "sender has already been set"
        );
    });
    info!("channel buffers successfully set");
}

#[wasm_bindgen]
pub fn send() -> Result<(), JsValue> {
    info!("send");

    let response = ServerToClientMessage::ErrorResponse(ErrorResponseMessage {
        id: RequestId(NonZeroU128::new(123).ok_or("impossible")?),
        message: "bla".to_owned(),
    });

    APPLICATION_STATE.with_borrow_mut(|state| {
        let Some(sender) = &mut state.sender else {
            return Err(JsValue::from_str("cannot run without a sender"));
        };

        sender.send(&response)
    })?;
    info!("message successfully sent");

    Ok(())
}
