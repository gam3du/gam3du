#![expect(dead_code, reason = "TODO WIP")]
//! Contains the types of messages that can be sent between endpoints.
//! Each endpoint creates a single mpsc channel in order to receive commands or events from multiple other endpoints.
//! The underlying framework will send the cloned channels' senders to the endpoints according to dependency graph.

use std::num::NonZeroU128;

pub(crate) enum Message {
    Request(RequestMessage),
    Response(ResponseMessage),
    ErrorResponse(ErrorResponseMessage),
    Event(EventMessage),
}

/// Name of a communication partner (Script, API or whatever we decide :) )
/// TODO Maybe these can be replaced by numeric IDs, if the framework provides a lookup-table
type EndpointName = String;

/// UUID to track and associate messages
type MessageId = NonZeroU128;

/// Asks the receiver to perform an operation and return a response containing the result.
pub(crate) struct RequestMessage {
    /// UUID used to track corresponding messages
    id: MessageId,
    /// Name of the sending/requesting endpoint
    /// This is required as the receiver will provide a single mpsc-channel which cannot distinguish between senders.
    // TODO maybe there an mpsc-implementation that can distinguish between multiple senders?
    source: EndpointName,
    /// Name of the communication partner who's responsible to respond to the request
    target: EndpointName,
    /// The actual function that shall be triggered
    command: String,
    /// The list of arguments to be passed to the called function
    arguments: serde_json::Value,
}

/// Contains the result of a requested operation
pub(crate) struct ResponseMessage {
    /// this shall match the id of the corresponding request
    id: MessageId,
    /// The result of the requested operation
    /// This might contain application errors if the request could not be fulfilled successfully
    result: serde_json::Value,
}

/// Indicates that a request could not be made into a proper function call.
/// This is a programming error of the requester.
/// Possible causes are: unknown recipient, unknown command, wrong argument configuration
/// This message type exists to not overload framework-related errors with actual application errors.
// TODO deserves a better name
pub(crate) struct ErrorResponseMessage {
    /// this shall match the id of the corresponding request
    id: MessageId,
    /// A readable description of what went wrong
    // TODO maybe this could be an enum of known error causes
    message: String,
}

/// Represents anything that happened without being triggered by a request.
pub(crate) struct EventMessage {
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
