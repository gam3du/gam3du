//! Provides commands to assist with more complex builds and deployments
#![cfg_attr(
    target_arch = "wasm32",
    allow(unused_crate_dependencies, reason = "no implementation for wasm")
)]
#[cfg(target_arch = "wasm32")]
fn main() {
    #![allow(clippy::panic, reason = "wasm is not supported")]
    panic!("WASM is not supported for xtask");
}

#[cfg(not(target_arch = "wasm32"))]
mod run_wasm;
#[cfg(not(target_arch = "wasm32"))]
mod util;

#[cfg(not(target_arch = "wasm32"))]
fn main() -> anyhow::Result<ExitCode> {
    use anyhow::Context;
    use pico_args::Arguments;
    use std::process::ExitCode;
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_indent(Some(0))
        .init();

    let args = Arguments::from_env();
    let shell = xshell::Shell::new().context("Couldn't create xshell shell")?;
    let root_dir = format!("{}/..", env!("CARGO_MANIFEST_DIR"));
    shell.change_dir(root_dir);

    run_wasm::run_wasm(&shell, args)?;
    Ok(ExitCode::SUCCESS)
}
