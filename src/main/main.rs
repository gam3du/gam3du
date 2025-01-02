//! Provides commands to assist with more complex builds and deployments
#![expect(
    clippy::print_stderr,
    reason = "This is a normal thing to do for a console application"
)]

mod ace;
mod tasks;
mod util;
mod wasm;

pub(crate) fn main() -> anyhow::Result<std::process::ExitCode> {
    use anyhow::Context;
    use log::error;
    use pico_args::Arguments;
    use std::process::ExitCode;
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .parse_default_env()
        .format_indent(Some(0))
        .init();

    let mut args = Arguments::from_env();
    let shell = xshell::Shell::new().context("Couldn't create xshell shell")?;
    // let root_dir = format!("{}/..", env!("CARGO_MANIFEST_DIR"));
    // shell.change_dir(root_dir);

    let Some(subcommand) = args.subcommand()? else {
        print_help();
        return Ok(ExitCode::SUCCESS);
    };

    // TODO add more tasks for "cargo update/clean/Cargo.toml-refresh"
    match subcommand.as_str() {
        "robot" => {
            tasks::robot_native::run(&shell, args)?;
            Ok(ExitCode::SUCCESS)
        }
        "robot-web" => {
            tasks::robot_web::run(&shell, args)?;
            Ok(ExitCode::SUCCESS)
        }
        unknown => {
            error!("unknown command: {unknown}");
            print_help();
            Ok(ExitCode::FAILURE)
        }
    }
}

fn print_help() {
    eprintln!("Start the robot application on the local machine:");
    eprintln!("\tcargo robot");
    eprintln!();

    eprintln!("Start the robot application as a web service:");
    eprintln!("\tcargo robot-web");
    eprintln!();
}
