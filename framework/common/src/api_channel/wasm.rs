use tracing::debug;
use wasm_rs_shared_channel::spsc;
use web_sys::{js_sys, wasm_bindgen::JsCast, DedicatedWorkerGlobalScope};

use crate::{
    api::ApiDescriptor,
    message::{ClientToServerMessage, ServerToClientMessage},
};

use super::ApiClientEndpoint;

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub struct WasmApiClientEndpoint {
    api: ApiDescriptor,
    /// Used to receive responses from the connected [`ApiServerEndpoint`]
    receiver: spsc::Receiver<ServerToClientMessage>,
}

impl WasmApiClientEndpoint {
    #[must_use]
    pub fn new(api: ApiDescriptor, receiver: spsc::Receiver<ServerToClientMessage>) -> Self {
        Self { api, receiver }
    }
}

impl ApiClientEndpoint for WasmApiClientEndpoint {
    #[must_use]
    fn api(&self) -> &ApiDescriptor {
        &self.api
    }

    fn send_to_server(&self, message: ClientToServerMessage) {
        debug!("send_to_server: {message:#?}");
        let bytes = bincode::serialize(&message).unwrap();
        debug!("send_to_server: {bytes:?}");

        let global = js_sys::global()
            .dyn_into::<DedicatedWorkerGlobalScope>()
            .unwrap();
        global.post_message(&bytes.into()).unwrap();
    }

    fn poll_response(&self) -> Option<ServerToClientMessage> {
        self.receiver.recv(None).unwrap()
    }
}
