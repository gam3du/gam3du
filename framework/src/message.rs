//! Contains the types of messages that can be sent between endpoints.
//! Each endpoint creates a single mpsc channel in order to receive commands or events from a single endpoints.

use rand::{thread_rng, Rng};

use crate::api::{Identifier, Value};
use std::{
    fmt::Display,
    num::{NonZeroU128, TryFromIntError},
};

/// Any message that can be sent from a client to a server
///
/// This might be extended in the future in order to query or cancel a pending request
pub enum ClientToServerMessage {
    Request(RequestMessage),
}

/// Any message that can be sent from a server to a client
pub enum ServerToClientMessage {
    Response(ResponseMessage),
    ErrorResponse(ErrorResponseMessage),
}

/// UUID to associate all messages with the initial request
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RequestId(pub NonZeroU128);

impl RequestId {
    #[must_use]
    fn new_random() -> Self {
        Self(thread_rng().r#gen())
    }
}

impl TryFrom<u128> for RequestId {
    type Error = TryFromIntError;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        value.try_into().map(Self)
    }
}

impl Display for RequestId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

/// Asks the receiver to perform an operation and return a response containing the result.
pub struct RequestMessage {
    /// UUID used to track corresponding messages
    pub id: RequestId,
    /// The name of the function that shall be triggered
    pub command: Identifier,
    /// The list of arguments to be passed to the called function
    pub arguments: Vec<Value>,
}

impl RequestMessage {
    #[must_use]
    pub(crate) fn new(command: Identifier, arguments: Vec<Value>) -> Self {
        Self {
            id: RequestId::new_random(),
            command,
            arguments,
        }
    }
}

/// Contains the result of a requested operation
// TODO find a better name to reflect a positive result value (e.g. `OkResultMessage` or `OkResponseMessage`)
pub struct ResponseMessage {
    /// this shall match the id of the corresponding request
    pub id: RequestId,
    /// The result of the requested operation
    /// This might contain application errors if the request could not be fulfilled successfully
    pub result: Value,
}

/// Indicates that a request could not be made into a proper function call.
///
/// This is a programming error of the requester.
/// Possible causes are: unknown recipient, unknown command, wrong argument configuration
/// This message type exists to not overload framework-related errors with actual application errors.
// TODO deserves a better name
pub struct ErrorResponseMessage {
    /// this shall match the id of the corresponding request
    pub id: RequestId,
    /// A readable description of what went wrong
    // TODO maybe this could be an enum of known error causes
    pub message: String,
}

impl From<ErrorResponseMessage> for ServerToClientMessage {
    fn from(value: ErrorResponseMessage) -> Self {
        Self::ErrorResponse(value)
    }
}

impl From<ResponseMessage> for ServerToClientMessage {
    fn from(value: ResponseMessage) -> Self {
        Self::Response(value)
    }
}

impl From<RequestMessage> for ClientToServerMessage {
    fn from(value: RequestMessage) -> Self {
        Self::Request(value)
    }
}

/// A little helper for the common case that a command's result might have a third `Future` state,
/// indicating the command has been accepted but the actual result will arrive later.
///
/// This allows execution of asynchronous commands without the communication overhead for trivial cases.
#[derive(Debug, Clone)]
#[must_use]
pub enum PendingResult<T, Error> {
    Ok(T),
    Pending,
    Error(Error),
}
