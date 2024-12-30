use tracing::{debug, trace};
use wasm_rs_shared_channel::spsc;
use web_sys::{js_sys, wasm_bindgen::JsCast, DedicatedWorkerGlobalScope, MessagePort};

use crate::{
    api::ApiDescriptor,
    message::{ClientToServerMessage, ServerToClientMessage},
};

use super::{ApiClientEndpoint, ApiServerEndpoint};

// type SendHandler = Box<dyn for<'bytes> Fn(&'bytes [u8]) + Send>;
type PollHandler = Box<dyn Fn() -> Option<Vec<u8>>>;

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub struct WasmApiClientEndpoint {
    api: ApiDescriptor,
    /// Used to receive responses from the connected [`ApiServerEndpoint`]
    receiver: spsc::Receiver<ServerToClientMessage>,
    // sender: MessagePort,
}

impl WasmApiClientEndpoint {
    #[must_use]
    pub fn new(
        api: ApiDescriptor,
        receiver: spsc::Receiver<ServerToClientMessage>,
        // sender: MessagePort,
    ) -> Self {
        Self {
            api,
            receiver,
            // sender,
        }
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
        // self.sender.post_message(&bytes.into()).unwrap();

        let global = js_sys::global()
            .dyn_into::<DedicatedWorkerGlobalScope>()
            .unwrap();
        global.post_message(&bytes.into()).unwrap();
    }

    fn poll_response(&self) -> Option<ServerToClientMessage> {
        // trace!("polling_response");
        let result = self.receiver.recv(None).unwrap();
        // trace!("poll_response: {result:?}");
        result
    }
}
