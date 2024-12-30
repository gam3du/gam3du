use gam3du_framework_common::{
    api::ApiDescriptor,
    api_channel::ApiServerEndpoint,
    message::{ClientToServerMessage, ServerToClientMessage},
};
use tracing::debug;
use web_sys::MessagePort;

use crate::wasm::APPLICATION_STATE;

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
        let bytes = bincode::serialize(&message).unwrap();
        debug!("send_to_client: {bytes:?}");
        self.sender.post_message(&bytes.into()).unwrap();
    }

    #[must_use]
    fn poll_request(&self) -> Option<ClientToServerMessage> {
        let message = APPLICATION_STATE.with_borrow_mut(|state| {
            // trace!("polling client message from queue");
            state.client_messages.pop_front()
        });

        message.map(|request_bytes| {
            debug!("received bytes from PythonWorker: {request_bytes:?}");
            let request: ClientToServerMessage = bincode::deserialize(&request_bytes).unwrap();
            debug!("forwarding request to plugin: {request:#?}");
            request
        })

        // if let Some(request_bytes) = (self.poll)() {
        //     trace!("received bytes from PythonWorker: {request_bytes:?}");
        //     let request: ClientToServerMessage = bincode::deserialize(&request_bytes).unwrap();
        //     trace!("received request from PythonWorker: {request:#?}");

        //     trace!("forwarding message to plugin");
        //     Some(request)
        // } else {
        //     None
        // }
    }

    #[must_use]
    fn api(&self) -> &ApiDescriptor {
        &self.api
    }
}
