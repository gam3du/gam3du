mod native;

// TODO maybe disable this for WASM or move into own platform specific crate?
// TODO maybe the entire channel stuff should not be in the common crate as there's too much implementation in them
pub use native::{native_channel, NativeApiClientEndpoint, NativeApiServerEndpoint};

use crate::{
    api::{ApiDescriptor, Identifier, Value},
    message::{
        ClientToServerMessage, ErrorResponseMessage, RequestId, ResponseMessage,
        ServerToClientMessage,
    },
};

/// Handles transmission of commands to [`ApiServerEndpoint`]s and provides methods for polling responses.
pub trait ApiClientEndpoint {
    fn send_to_server(&self, message: ClientToServerMessage);

    #[must_use]
    fn api(&self) -> &ApiDescriptor;

    #[must_use]
    fn send_command(&self, command: Identifier, arguments: Vec<Value>) -> RequestId;

    #[must_use]
    fn poll_response(&self) -> Option<ServerToClientMessage>;
}

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub trait ApiServerEndpoint {
    fn send_to_client(&self, message: ServerToClientMessage);

    fn send_error(&mut self, id: RequestId, message: String) {
        let response = ErrorResponseMessage { id, message };
        self.send_to_client(response.into());
    }

    fn send_response(&self, id: RequestId, result: Value) {
        let response = ResponseMessage { id, result };
        self.send_to_client(response.into());
    }

    #[must_use]
    fn poll_request(&self) -> Option<ClientToServerMessage>;

    #[must_use]
    fn api(&self) -> &ApiDescriptor;
}
