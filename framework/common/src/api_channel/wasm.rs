use tracing::trace;
use wasm_rs_shared_channel::spsc;

use crate::{
    api::ApiDescriptor,
    message::{ClientToServerMessage, ServerToClientMessage},
};

use super::{ApiClientEndpoint, ApiServerEndpoint};

type SendHandler = Box<dyn for<'bytes> Fn(&'bytes [u8]) + Send>;

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub struct WasmApiClientEndpoint {
    api: ApiDescriptor,
    /// Used to send responses to the connected [`ApiClientEndpoint`]
    receiver: spsc::Receiver<ServerToClientMessage>,
    send: SendHandler,
}

impl WasmApiClientEndpoint {
    #[must_use]
    pub fn new(
        api: ApiDescriptor,
        receiver: spsc::Receiver<ServerToClientMessage>,
        send: SendHandler,
    ) -> Self {
        Self {
            api,
            receiver,
            send,
        }
    }
}

impl ApiClientEndpoint for WasmApiClientEndpoint {
    #[must_use]
    fn api(&self) -> &ApiDescriptor {
        &self.api
    }

    fn send_to_server(&self, message: ClientToServerMessage) {
        (self.send)(&bincode::serialize(&message).unwrap());
    }

    fn poll_response(&self) -> Option<ServerToClientMessage> {
        self.receiver.recv(None).unwrap()
    }
}

/// Provides methods for polling on requests from an [`ApiClientEndpoint`] and sending back responses.
pub struct WasmApiServerEndpoint {
    api: ApiDescriptor,
    /// Used to send responses to the connected [`ApiClientEndpoint`]
    sender: spsc::Sender<ServerToClientMessage>,
    poll: Box<dyn Fn() -> Option<Vec<u8>>>,
}

impl WasmApiServerEndpoint {
    #[must_use]
    pub fn new(
        api: ApiDescriptor,
        sender: spsc::Sender<ServerToClientMessage>,
        poll: Box<dyn Fn() -> Option<Vec<u8>>>,
    ) -> Self {
        Self { api, sender, poll }
    }
}

impl ApiServerEndpoint for WasmApiServerEndpoint {
    fn send_to_client(&self, message: ServerToClientMessage) {
        tracing::trace!("forwarding message to Python Worker");
        self.sender.send(&message).unwrap();
    }

    #[must_use]
    fn poll_request(&self) -> Option<ClientToServerMessage> {
        if let Some(request_bytes) = (self.poll)() {
            let request = bincode::deserialize(&request_bytes).unwrap();
            trace!("received request from PythonWorker: {request:#?}");

            trace!("forwarding message to plugin");
            Some(ClientToServerMessage::Request(request))
        } else {
            None
        }
    }

    #[must_use]
    fn api(&self) -> &ApiDescriptor {
        &self.api
    }
}
