use gam3du_framework_common::{
    api::ApiDescriptor,
    api_channel::ApiServerEndpoint,
    message::{ClientToServerMessage, ServerToClientMessage},
};
use tracing::debug;
use web_sys::MessagePort;

use crate::APPLICATION_STATE;

/// Provides methods for polling on requests from an [`ApiClientEndpoint`] and sending back responses.
pub(crate) struct WasmApiServerEndpoint {
    api: ApiDescriptor,
    /// Used to send responses to the connected [`ApiClientEndpoint`]
    sender: MessagePort,
}

impl WasmApiServerEndpoint {
    #[must_use]
    pub(crate) fn new(api: ApiDescriptor, sender: MessagePort) -> Self {
        Self { api, sender }
    }
}

impl ApiServerEndpoint for WasmApiServerEndpoint {
    fn send_to_client(&self, message: ServerToClientMessage) {
        debug!("send_to_client: {message:#?}");
        let bytes = bincode::serde::encode_to_vec(&message, bincode::config::standard()).unwrap();
        debug!("send_to_client: {bytes:?}");
        self.sender.post_message(&bytes.into()).unwrap();
    }

    fn poll_request(&self) -> Option<ClientToServerMessage> {
        let message = APPLICATION_STATE.with_borrow_mut(|state| state.client_messages.pop_front());

        message.map(|request_bytes| {
            debug!("received bytes from PythonWorker: {request_bytes:?}");
            let request: ClientToServerMessage =
                bincode::serde::decode_from_slice(&request_bytes, bincode::config::standard())
                    .unwrap()
                    .0;
            debug!("forwarding request to plugin: {request:#?}");
            request
        })
    }

    fn api(&self) -> &ApiDescriptor {
        &self.api
    }
}
