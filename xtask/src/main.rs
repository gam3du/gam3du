//! Provides commands to assist with more complex builds and deployments

use anyhow::Context;
use pico_args::Arguments;
use std::process::ExitCode;

mod run_wasm;
mod util;

fn main() -> anyhow::Result<ExitCode> {
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
