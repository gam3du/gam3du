//! Contains all the building blocks to specify an API and perform reflection thereon.
#![expect(
    dead_code,
    clippy::panic,
    clippy::unwrap_used,
    reason = "TODO fix after experimentation phase"
)]

use std::{
    fmt::Display,
    ops::Range,
    sync::mpsc::{Receiver, Sender, TryRecvError},
};

use rand::{thread_rng, Rng};
use serde::{Deserialize, Serialize};

use crate::message::{
    ClientToServerMessage, MessageId, RequestMessage, ResponseMessage, ServerToClientMessage,
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RichText(pub String);

/// A technical name of an element (api, function, parameter, …).
///
/// For compatibility reasons only the ASCII-characters `a-z`, `0-9` and ` ` are allowed.
/// No uppercase letters, dashes/minus, underscores, dots, etc. are permitted.
/// An single space serves as a separator between words; they may not appear
/// at the start or end of an identifier and no two spaces may be adjacent
///
/// The binding generators will perform some name mangling on those identifiers
/// to make sure they fit into the target ecosystem. This is why a space was chosen:
/// It emphasizes best that such a name mangling _must_ occur and is a desired behavior
/// as a space is rarely accepted within identifiers.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Identifier(pub String);

impl Display for Identifier {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Api {
    /// technical name of this API
    pub name: Identifier,
    /// a single-line explanation what this api is for
    pub caption: RichText,
    /// a multi-line explanation what this api is for
    pub description: RichText,
    /// List of all functions this API provides
    pub functions: Vec<FunctionDescriptor>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FunctionDescriptor {
    /// technical name of this function
    pub name: Identifier,
    /// a single-line explanation what this function is for
    pub caption: RichText,
    /// a multi-line explanation what this function is for
    pub description: RichText,
    /// List of all parameters this function requires
    pub parameters: Vec<ParameterDescriptor>,
    /// List of all parameters this function requires
    pub returns: Option<ParameterDescriptor>,
}

/// Description of a function parameter or return value
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ParameterDescriptor {
    /// technical name of this function parameter
    /// While the return value _may_ have a name, it's generally not used by most apis. Using "return" is recommended
    pub name: Identifier,
    /// a single-line explanation what this parameter is for
    pub caption: RichText,
    /// a multi-line explanation what this parameter is for
    pub description: RichText,
    /// Data type of this parameter
    pub typ: TypeDescriptor,
    /// If the parameter is omitted, use a default value
    pub default: Option<Value>,
}

/// Describes the set of valid values for a parameter or variable.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TypeDescriptor {
    /// Any integer value within the defined range. Unsigned values will typically have a lower bound of `0`
    /// Both bounds are required to fit into `i48` or `u48`.
    Integer(Range<i64>),
    Float,
    Boolean,
    String,
    List(Box<TypeDescriptor>),
}

impl TypeDescriptor {
    pub const INTEGER_BITS: u32 = 48;
    pub const MAX_UNSIGNED_INTEGER: u64 = (1 << Self::INTEGER_BITS) - 1;
    pub const MAX_SIGNED_INTEGER: i64 = (1 << (Self::INTEGER_BITS - 1)) - 1;
    pub const MIN_INTEGER: i64 = -(1 << (Self::INTEGER_BITS - 1));
}

/// A value for a parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Integer(i64),
    Float(f32),
    Boolean(bool),
    String(String),
    List(Box<Value>),
}

/// Handles transmission of commands to [`ApiServerEndpoint`]s and provides methods for polling responses.
pub(crate) struct ApiClientEndpoint {
    /// Used to send requests to the connected [`ApiServerEndpoint`]
    sender: Sender<ClientToServerMessage>,
    /// Used poll for responses from the the connected [`ApiServerEndpoint`]
    receiver: Receiver<ServerToClientMessage>,
}

impl ApiClientEndpoint {
    pub(crate) fn new(
        receiver: Receiver<ServerToClientMessage>,
        sender: Sender<ClientToServerMessage>,
    ) -> Self {
        Self { sender, receiver }
    }

    pub(crate) fn send_command(
        &mut self,
        command: Identifier,
        arguments: serde_json::Value,
    ) -> MessageId {
        let id = thread_rng().r#gen();

        let request = RequestMessage {
            id,
            command,
            arguments,
        };

        self.send_to_server(request);

        id
    }

    fn send_to_server(&mut self, message: impl Into<ClientToServerMessage>) {
        self.sender.send(message.into()).unwrap();
    }

    pub(crate) fn poll_response(&mut self) -> Option<ServerToClientMessage> {
        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(error @ TryRecvError::Disconnected) => panic!("{error}"),
        }
    }
}

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub(crate) struct ApiServerEndpoint {
    /// Used poll for requests from the the connected [`ApiClientEndpoint`]
    receiver: Receiver<ClientToServerMessage>,
    /// Used to send responses to the connected [`ApiClientEndpoint`]
    sender: Sender<ServerToClientMessage>,
}

impl ApiServerEndpoint {
    pub(crate) fn new(
        receiver: Receiver<ClientToServerMessage>,
        sender: Sender<ServerToClientMessage>,
    ) -> Self {
        Self { receiver, sender }
    }

    pub(crate) fn send_response(&mut self, id: MessageId, result: serde_json::Value) {
        let response = ResponseMessage { id, result };
        self.send_to_client(response);
    }

    fn send_to_client(&mut self, message: impl Into<ServerToClientMessage>) {
        self.sender.send(message.into()).unwrap();
    }

    pub(crate) fn poll_request(&mut self) -> Option<ClientToServerMessage> {
        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(error @ TryRecvError::Disconnected) => panic!("{error}"),
        }
    }
}
