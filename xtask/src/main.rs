//! Provides commands to assist with more complex builds and deployments
#![cfg_attr(
    target_family = "wasm",
    allow(
        unused_crate_dependencies,
        reason = "this crate is not supposed to be built on wasm"
    )
)]

#[cfg(target_family = "wasm")]
fn main() {}

#[cfg(not(target_family = "wasm"))]
mod ace;
#[cfg(not(target_family = "wasm"))]
mod util;
#[cfg(not(target_family = "wasm"))]
mod wasm;

#[cfg(not(target_family = "wasm"))]
#[path = "tasks/robot_web.rs"]
mod robot_web;
#[cfg(not(target_family = "wasm"))]
#[path = "tasks/run_wasm.rs"]
mod run_wasm;

#[cfg(not(target_family = "wasm"))]
fn main() -> anyhow::Result<std::process::ExitCode> {
    use anyhow::Context;
    use pico_args::Arguments;
    use std::process::ExitCode;
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
        "robot-web" => {
            robot_web::run(&shell, args)?;
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
