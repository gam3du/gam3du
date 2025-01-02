use crate::main::{
    ace,
    util::copy_content,
    wasm::{build_wasm, check_wasm_programs, start_webserver},
};
use pico_args::Arguments;
use std::{fs, path::Path};
use xshell::Shell;

pub(crate) fn run(shell: &Shell, mut args: Arguments) -> anyhow::Result<()> {
    let no_serve = args.contains("--no-serve");
    let is_release = args.contains("--release");

    check_wasm_programs(no_serve)?;

    let target_dir = Path::new("target/generated");

    copy_content(
        shell,
        Path::new("applications/robot/web/static"),
        target_dir,
    )?;

    let cargo_args = args.finish();

    build_wasm(
        shell,
        "application-robot-web-runtime-python",
        "application_robot_web_runtime_python",
        is_release,
        &target_dir.join("runtime-python/wasm"),
        &cargo_args,
    )?;

    build_wasm(
        shell,
        "application-robot-web-main",
        "application_robot_web_main",
        is_release,
        &target_dir.join("wasm"),
        &cargo_args,
    )?;

    ace::download(shell, &target_dir.join("ace"))?;

    let index_html = target_dir.join("index.html");
    let index_html_str = fs::read_to_string(&index_html)?;
    let index_html_str = index_html_str.replace(
        "{code}",
        include_str!("../../../applications/robot/python/control/robot.py"),
    );
    fs::write(&index_html, index_html_str)?;

    if !no_serve {
        start_webserver(shell, target_dir)?;
    }

    Ok(())
}
