//! This is the crate's entry point for WASM targets which ensures that only non-WASM builds will
//! ever see an actual implementation.
//!
//! WASM-builds will always compile an empty crate. This ensures the IDE won't show any distracting
//! errors about not being able to build the native implementation for the web.
#![allow(
    unused_crate_dependencies,
    reason = "this crate is not meant to be built on wasm"
)]

#[cfg(target_family = "wasm")]
fn main() {
    // this crate is not meant to be built on wasm targets
}

#[cfg(not(target_family = "wasm"))]
#[cfg_attr(not(target_family = "wasm"), path = "main/main.rs")]
mod main;

#[cfg(not(target_family = "wasm"))]
fn main() -> anyhow::Result<std::process::ExitCode> {
    main::main()
}
