use std::path::Path;

use crate::{
    ace,
    util::copy_content,
    wasm::{build_wasm, check_wasm_programs, start_webserver},
};
use pico_args::Arguments;
use xshell::Shell;

pub(crate) fn run(shell: &Shell, mut args: Arguments) -> anyhow::Result<()> {
    let no_serve = args.contains("--no-serve");
    let is_release = args.contains("--release");

    check_wasm_programs(no_serve)?;

    let target_dir = Path::new("target/generated");

    copy_content(shell, Path::new("static"), target_dir)?;

    let cargo_args = args.finish();

    build_wasm(
        shell,
        "lib-wasm-tools",
        "lib_wasm_tools",
        is_release,
        &target_dir.join("wasm-tools/wasm"),
        &cargo_args,
    )?;

    build_wasm(
        shell,
        "runtime-python-wasm",
        "runtime_python_wasm",
        is_release,
        &target_dir.join("runtime-python/wasm"),
        &cargo_args,
    )?;

    build_wasm(
        shell,
        "application-robot",
        "application-robot",
        is_release,
        &target_dir.join("application/wasm"),
        &cargo_args,
    )?;

    ace::download(&target_dir.join("ace"))?;

    if !no_serve {
        start_webserver(shell, target_dir)?;
    }

    Ok(())
}
