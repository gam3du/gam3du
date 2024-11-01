use std::process::ExitCode;

use anyhow::Context;
use pico_args::Arguments;

mod run_wasm;
mod util;

fn main() -> anyhow::Result<ExitCode> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_indent(Some(0))
        .init();

    let mut args = Arguments::from_env();

    // -- Shell Creation --

    let shell = xshell::Shell::new().context("Couldn't create xshell shell")?;
    shell.change_dir(String::from(env!("CARGO_MANIFEST_DIR")) + "/..");

    run_wasm::run_wasm(shell, args)?;
    Ok(ExitCode::SUCCESS)
}
