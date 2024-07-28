// false positives with multiple crates
#![allow(unused_crate_dependencies)]
// TODO remove and fix before release
#![allow(missing_docs)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unwrap_used)]
#![allow(clippy::expect_used)]
#![allow(clippy::panic)]

pub mod bindgen;
mod runner;

pub use runner::runner;
