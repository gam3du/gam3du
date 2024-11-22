#![allow(missing_docs, reason = "TODO remove before release")]
// #![expect(
//     // clippy::cast_precision_loss,
//     // clippy::expect_used,
//     // clippy::indexing_slicing,
//     // clippy::missing_errors_doc,
//     // clippy::missing_panics_doc,
//     // clippy::panic,
//     // clippy::print_stdout,
//     // clippy::todo,
//     // clippy::unwrap_used,
//     reason = "TODO remove before release"
// )]

// this is indirectly required by the `rand` crate to support WASM
use getrandom as _;

pub mod api;
pub mod api_channel;
pub mod event;
pub mod message;
pub mod module;
