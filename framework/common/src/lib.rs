#![allow(
    missing_docs,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    reason = "TODO remove before release"
)]

// this is indirectly required by the `rand` crate to support WASM
use getrandom as _;

pub mod api;
pub mod api_channel;
pub mod event;
pub mod message;
pub mod module;
