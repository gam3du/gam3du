#![expect(dead_code, reason = "TODO WIP")]
//! Contains the types of messages that can be sent between endpoints.
//! Each endpoint creates a single mpsc channel in order to receive commands or events from multiple other endpoints.
//! The underlying framework will send the cloned channels' senders to the endpoints according to dependency graph.

use std::num::NonZeroU128;

use crate::api::{Identifier, Value};

pub enum ClientToServerMessage {
    Request(RequestMessage),
}

pub enum ServerToClientMessage {
    Response(ResponseMessage),
    ErrorResponse(ErrorResponseMessage),
    Event(EventMessage),
}

// pub(crate) enum Message {
//     Request(RequestMessage),
//     Response(ResponseMessage),
//     ErrorResponse(ErrorResponseMessage),
//     Event(EventMessage),
// }

/// Name of a communication partner (Script, API or whatever we decide :) )
/// TODO Maybe these can be replaced by numeric IDs, if the framework provides a lookup-table
type EndpointName = Identifier;

/// UUID to track and associate messages
// TODO this should rather be a `CallId` or similar because a single call may involve several messages (request, response, error, …)
pub type MessageId = NonZeroU128;

/// Asks the receiver to perform an operation and return a response containing the result.
pub struct RequestMessage {
    /// UUID used to track corresponding messages
    pub id: MessageId,
    // /// Name of the sending/requesting endpoint
    // /// This is required as the receiver will provide a single mpsc-channel which cannot distinguish between senders.
    // // TODO maybe there an mpsc-implementation that can distinguish between multiple senders?
    // source: EndpointName,
    // /// Name of the communication partner who's responsible to respond to the request
    // target: EndpointName,
    /// The actual function that shall be triggered
    pub command: Identifier,
    /// The list of arguments to be passed to the called function
    pub arguments: Vec<Value>,
}

/// Contains the result of a requested operation
// TODO find a better name to reflect a positive result value (e.g. `OkResultMessage` or `OkResponseMessage`)
pub struct ResponseMessage {
    /// this shall match the id of the corresponding request
    pub id: MessageId,
    /// The result of the requested operation
    /// This might contain application errors if the request could not be fulfilled successfully
    pub result: serde_json::Value,
}

/// Indicates that a request could not be made into a proper function call.
/// This is a programming error of the requester.
/// Possible causes are: unknown recipient, unknown command, wrong argument configuration
/// This message type exists to not overload framework-related errors with actual application errors.
// TODO deserves a better name
pub struct ErrorResponseMessage {
    /// this shall match the id of the corresponding request
    pub id: MessageId,
    /// A readable description of what went wrong
    // TODO maybe this could be an enum of known error causes
    pub message: String,
}

/// Represents anything that happened without being triggered by a request.
pub struct EventMessage {
    /// UUID used to track the event for debugging and detecting forwarding loops
    id: MessageId,
    /// Name of the sending/requesting endpoint
    /// This is required as the receiver will provide a single mpsc-channel which cannot distinguish between senders.
    source: EndpointName,
    /// What kind of event happened
    name: String,
    /// The payload of this event
    content: serde_json::Value,
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
impl From<EventMessage> for ServerToClientMessage {
    fn from(value: EventMessage) -> Self {
        Self::Event(value)
    }
}
impl From<RequestMessage> for ClientToServerMessage {
    fn from(value: RequestMessage) -> Self {
        Self::Request(value)
    }
}
