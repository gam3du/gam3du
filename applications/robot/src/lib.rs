//! Contains common types for both, the native and the web implementation of this application
//! Also executes the build scripts that generate the Python bindings.

// #![allow(missing_docs, reason = "TODO remove before release")]
// #![expect(
//     clippy::unwrap_used,
//     // clippy::expect_used,
//     clippy::todo,
//     clippy::missing_errors_doc,
//     clippy::missing_panics_doc,
//     // clippy::unwrap_in_result,

//     reason = "TODO remove before release"
// )]

// // mod api_endpoint;
// mod error;

/// Name to be used as readable window title for this application
pub const APPLICATION_TITLE: &str = "Robot";

// ///////////////////////// native section /////////////////////////

// #[cfg(not(target_family = "wasm"))]
// mod native;

// #[cfg(not(target_family = "wasm"))]
// fn main() {
//     todo!("restore native implementation from main branch");
// }

// ///////////////////////// WASM section /////////////////////////

// // #[cfg(target_family = "wasm")]
// // mod wasm;

// // #[cfg(target_family = "wasm")]
// // pub use wasm::{init, start};

// // #[cfg(target_family = "wasm")]
// // #[cfg_attr(target_family = "wasm", wasm_bindgen::prelude::wasm_bindgen(start))]
// // fn main() -> Result<(), wasm_bindgen::JsValue> {
// //     gam3du_framework::init_logger();
// //     tracing::info!("application loaded");
// //     Ok(())
// // }
