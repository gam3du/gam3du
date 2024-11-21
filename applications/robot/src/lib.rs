#![allow(missing_docs, reason = "TODO remove before release")]
#![expect(
    clippy::unwrap_used,
    // clippy::expect_used,
    clippy::todo,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,

    reason = "TODO remove before release"
)]

///////////////////////// native section /////////////////////////
mod error;
///
#[cfg(not(target_family = "wasm"))]
mod native;

#[cfg(not(target_family = "wasm"))]
fn main() {
    todo!("restore native implementation from main branch");
}

///////////////////////// WASM section /////////////////////////

#[cfg(target_family = "wasm")]
mod wasm;

#[cfg(target_family = "wasm")]
pub use wasm::{connect_api_client, init, start};

#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(start))]
fn main() -> Result<(), wasm_bindgen::JsValue> {
    init()
}
