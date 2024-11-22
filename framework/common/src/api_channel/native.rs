#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "TODO fix after experimentation phase"
)]

use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use crate::{
    api::{ApiDescriptor, Identifier, Value},
    message::{ClientToServerMessage, RequestId, RequestMessage, ServerToClientMessage},
};

use super::{ApiClientEndpoint, ApiServerEndpoint};

/// creates a connected pair of endpoints
#[must_use]
pub fn channel(api: ApiDescriptor) -> (NativeApiClientEndpoint, NativeApiServerEndpoint) {
    let (script_to_engine_sender, script_to_engine_receiver) = mpsc::channel();
    let (engine_to_script_sender, engine_to_script_receiver) = mpsc::channel();

    let server_endpoint = NativeApiServerEndpoint::new(
        api.clone(),
        script_to_engine_receiver,
        engine_to_script_sender,
    );

    let client_endpoint =
        NativeApiClientEndpoint::new(api, script_to_engine_sender, engine_to_script_receiver);

    (client_endpoint, server_endpoint)
}

/// Handles transmission of commands to [`ApiServerEndpoint`]s and provides methods for polling responses.
pub struct NativeApiClientEndpoint {
    api: ApiDescriptor,
    /// Used to send requests to the connected [`ApiServerEndpoint`]
    sender: Sender<ClientToServerMessage>,
    /// Used poll for responses from the the connected [`ApiServerEndpoint`]
    receiver: Receiver<ServerToClientMessage>,
}

impl NativeApiClientEndpoint {
    #[must_use]
    pub fn new(
        api: ApiDescriptor,
        sender: Sender<ClientToServerMessage>,
        receiver: Receiver<ServerToClientMessage>,
    ) -> Self {
        Self {
            api,
            sender,
            receiver,
        }
    }
}

impl ApiClientEndpoint for NativeApiClientEndpoint {
    fn send_to_server(&self, message: ClientToServerMessage) {
        self.sender.send(message).unwrap_or_else(|_| {
            panic!(
                "failed to send message to disconnected api server endpoint: `{}`",
                self.api.name
            )
        });
    }

    #[must_use]
    fn api(&self) -> &ApiDescriptor {
        &self.api
    }

    #[must_use]
    fn send_command(&self, command: Identifier, arguments: Vec<Value>) -> RequestId {
        let request = RequestMessage::new(command, arguments);
        let id = request.id;
        self.send_to_server(request.into());
        id
    }

    #[must_use]
    fn poll_response(&self) -> Option<ServerToClientMessage> {
        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(error @ TryRecvError::Disconnected) => panic!("{error}"),
        }
    }
}

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub struct NativeApiServerEndpoint {
    api: ApiDescriptor,
    /// Used poll for requests from the the connected [`ApiClientEndpoint`]
    receiver: Receiver<ClientToServerMessage>,
    /// Used to send responses to the connected [`ApiClientEndpoint`]
    sender: Sender<ServerToClientMessage>,
}

impl NativeApiServerEndpoint {
    #[must_use]
    pub fn new(
        api: ApiDescriptor,
        receiver: Receiver<ClientToServerMessage>,
        sender: Sender<ServerToClientMessage>,
    ) -> Self {
        Self {
            api,
            receiver,
            sender,
        }
    }
}

impl ApiServerEndpoint for NativeApiServerEndpoint {
    fn send_to_client(&self, message: ServerToClientMessage) {
        self.sender.send(message).unwrap();
    }

    #[must_use]
    fn poll_request(&self) -> Option<ClientToServerMessage> {
        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(error @ TryRecvError::Disconnected) => panic!("{error}"),
        }
    }

    #[must_use]
    fn api(&self) -> &ApiDescriptor {
        &self.api
    }
}
