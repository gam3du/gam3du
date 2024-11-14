//! Provides commands to assist with more complex builds and deployments

use std::process::ExitCode;

mod ace;
#[cfg(not(target_arch = "wasm32"))]
#[path = "tasks/run_wasm.rs"]
mod run_wasm;
#[cfg(not(target_arch = "wasm32"))]
mod util;
mod wasm;
#[cfg(not(target_arch = "wasm32"))]
#[path = "tasks/web.rs"]
mod web;

fn main() -> anyhow::Result<ExitCode> {
    use anyhow::Context;
    use pico_args::Arguments;
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_indent(Some(0))
        .init();

    let mut args = Arguments::from_env();
    let shell = xshell::Shell::new().context("Couldn't create xshell shell")?;
    let root_dir = format!("{}/..", env!("CARGO_MANIFEST_DIR"));
    shell.change_dir(root_dir);

    let Some(subcommand) = args.subcommand()? else {
        todo!("print help");
    };

    match subcommand.as_str() {
        "web" => {
            web::run(&shell, args)?;
            Ok(ExitCode::SUCCESS)
        }
        "web_legacy" => {
            run_wasm::run(&shell, args)?;
            Ok(ExitCode::SUCCESS)
        }
        unknown => {
            todo!("unknown command: {unknown}");
        }
    }
}
