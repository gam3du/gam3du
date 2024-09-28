//! Contains all the building blocks to specify an API and perform reflection thereon.
#![expect(
    clippy::panic,
    clippy::unwrap_used,
    reason = "TODO fix after experimentation phase"
)]

use crate::message::{
    ClientToServerMessage, ErrorResponseMessage, RequestId, RequestMessage, ResponseMessage,
    ServerToClientMessage,
};
use indexmap::IndexMap as HashMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    fmt::Display,
    ops::Range,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
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
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct Identifier(pub Cow<'static, str>);

impl Display for Identifier {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, formatter)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ApiDescriptor {
    /// technical name of this API
    pub name: Identifier,
    /// a single-line explanation what this api is for
    pub caption: RichText,
    /// a multi-line explanation what this api is for
    pub description: RichText,
    /// List of all functions this API provides
    pub functions: HashMap<Identifier, FunctionDescriptor>,
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
    #[serde(rename = "type")]
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

/// creates a connected pair of endpoints
#[must_use]
pub fn channel(api: ApiDescriptor) -> (ApiClientEndpoint, ApiServerEndpoint) {
    let (script_to_engine_sender, script_to_engine_receiver) = mpsc::channel();
    let (engine_to_script_sender, engine_to_script_receiver) = mpsc::channel();

    let server_endpoint = ApiServerEndpoint::new(
        api.clone(),
        script_to_engine_receiver,
        engine_to_script_sender,
    );

    let client_endpoint =
        ApiClientEndpoint::new(api, script_to_engine_sender, engine_to_script_receiver);

    (client_endpoint, server_endpoint)
}

/// A value for a parameter.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Value {
    Unit,
    Integer(i64),
    Float(f32),
    Boolean(bool),
    String(String),
    List(Box<Value>),
}

/// Handles transmission of commands to [`ApiServerEndpoint`]s and provides methods for polling responses.
pub struct ApiClientEndpoint {
    api: ApiDescriptor,
    /// Used to send requests to the connected [`ApiServerEndpoint`]
    sender: Sender<ClientToServerMessage>,
    /// Used poll for responses from the the connected [`ApiServerEndpoint`]
    receiver: Receiver<ServerToClientMessage>,
}

impl ApiClientEndpoint {
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

    pub fn send_to_server(&self, message: impl Into<ClientToServerMessage>) {
        self.sender.send(message.into()).unwrap();
    }

    #[must_use]
    pub fn api(&self) -> &ApiDescriptor {
        &self.api
    }

    #[must_use]
    pub fn send_command(&self, command: Identifier, arguments: Vec<Value>) -> RequestId {
        let request = RequestMessage::new(command, arguments);
        let id = request.id;
        self.send_to_server(request);
        id
    }

    #[must_use]
    pub fn poll_response(&self) -> Option<ServerToClientMessage> {
        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(error @ TryRecvError::Disconnected) => panic!("{error}"),
        }
    }
}

/// Provides methods for polling on requests from a [`ApiClientEndpoint`]s and sending back responses.
pub struct ApiServerEndpoint {
    api: ApiDescriptor,
    /// Used poll for requests from the the connected [`ApiClientEndpoint`]
    receiver: Receiver<ClientToServerMessage>,
    /// Used to send responses to the connected [`ApiClientEndpoint`]
    sender: Sender<ServerToClientMessage>,
}

impl ApiServerEndpoint {
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

    pub fn send_to_client(&mut self, message: impl Into<ServerToClientMessage>) {
        self.sender.send(message.into()).unwrap();
    }

    #[expect(
        clippy::needless_pass_by_value,
        reason = "passing a reference would make it impossible to pass a `String` without cloning"
    )]
    pub fn send_error(&mut self, id: RequestId, message: impl ToString) {
        let response = ErrorResponseMessage {
            id,
            message: message.to_string(),
        };
        self.send_to_client(response);
    }

    pub fn send_response(&mut self, id: RequestId, result: Value) {
        let response = ResponseMessage { id, result };
        self.send_to_client(response);
    }

    pub fn poll_request(&mut self) -> Option<ClientToServerMessage> {
        match self.receiver.try_recv() {
            Ok(message) => Some(message),
            Err(TryRecvError::Empty) => None,
            Err(error @ TryRecvError::Disconnected) => panic!("{error}"),
        }
    }

    #[must_use]
    pub fn api(&self) -> &ApiDescriptor {
        &self.api
    }
}
