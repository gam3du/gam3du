mod native;

pub use native::{channel, NativeApiClientEndpoint, NativeApiServerEndpoint};

use crate::{
    api::{ApiDescriptor, Identifier, Value},
    message::{
        ClientToServerMessage, ErrorResponseMessage, RequestId, ResponseMessage,
        ServerToClientMessage,
    },
};

/// Handles transmission of commands to [`ApiServerEndpoint`]s and provides methods for polling responses.
pub trait ApiClientEndpoint {
    fn send_to_server(&self, message: impl Into<ClientToServerMessage>);

    #[must_use]
    fn api(&self) -> &ApiDescriptor;

    #[must_use]
    fn send_command(&self, command: Identifier, arguments: Vec<Value>) -> RequestId;

    #[must_use]
    fn poll_response(&self) -> Option<ServerToClientMessage>;
}

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub trait ApiServerEndpoint {
    fn send_to_client(&self, message: impl Into<ServerToClientMessage>);

    fn send_error(&mut self, id: RequestId, message: impl Into<String>) {
        let response = ErrorResponseMessage {
            id,
            message: message.into(),
        };
        self.send_to_client(response);
    }

    fn send_response(&self, id: RequestId, result: Value) {
        let response = ResponseMessage { id, result };
        self.send_to_client(response);
    }

    #[must_use]
    fn poll_request(&self) -> Option<ClientToServerMessage>;

    #[must_use]
    fn api(&self) -> &ApiDescriptor;
}
