use std::{ffi::OsString, path::Path};

use anyhow::{bail, Context};
use xshell::Shell;

use crate::util::{check_all_programs, Program};

const WASM_BINDGEN: Program = Program {
    crate_name: "wasm-bindgen-cli",
    binary_name: "wasm-bindgen",
};

const SIMPLE_HTTP_SERVER: Program = Program {
    crate_name: "simple-http-server",
    binary_name: "simple-http-server",
};

pub(crate) fn check_wasm_programs(no_serve: bool) -> anyhow::Result<()> {
    let programs_needed: &[_] = if no_serve {
        &[WASM_BINDGEN]
    } else {
        &[WASM_BINDGEN, SIMPLE_HTTP_SERVER]
    };

    check_all_programs(programs_needed)
}

pub(crate) fn build_wasm(
    shell: &Shell,
    package_name: &str,
    binary_name: &str,
    is_release: bool,
    target_path: &Path,
    cargo_args: &[OsString],
) -> anyhow::Result<()> {
    let release_flag: &[_] = if is_release { &["--release"] } else { &[] };
    let output_dir = if is_release { "release" } else { "debug" };

    log::info!("building robot application");

    shell.change_dir("workspace-web");

    xshell::cmd!(
        shell,
        "cargo build --jobs -1 --package {package_name} {release_flag...}"
    )
    .args(cargo_args)
    // .quiet()
    .run()
    .context("Failed to build {package_name} as wasm32")?;

    // back to project root
    shell.change_dir("..");

    // TODO figure out why applications have no name mangling while libs have
    let mangled_name = binary_name; // package_name.replace('-', "_");
    let target_dir = target_path
        .parent()
        .ok_or_else(|| anyhow::format_err!("cannot get directory of {}", target_path.display()))?;
    let target_file = target_path
        .file_name()
        .ok_or_else(|| anyhow::format_err!("cannot get file name of {}", target_path.display()))?;

    xshell::cmd!(
        shell,
        "wasm-bindgen target/wasm32-unknown-unknown/{output_dir}/{mangled_name}.wasm --target web --no-typescript --out-dir {target_dir} --out-name {target_file}"
    )
    // .quiet()
    .run()
    .context("Failed to run bindgen for {package_name}")?;

    Ok(())
}

pub(crate) fn start_webserver(shell: &Shell, root_path: &Path) -> anyhow::Result<()> {
    log::info!("serving on port 8000");

    // Explicitly specify the IP address to 127.0.0.1 since otherwise simple-http-server will
    // print http://0.0.0.0:8000 as url which is not a secure context and thus doesn't allow
    // running WebGPU!
    xshell::cmd!(
            shell,
            "simple-http-server {root_path} --compress wasm,html,js --coep --coop --ip 127.0.0.1 --index --nocache"
        )
        .quiet()
        .run()
        .context("Failed to simple-http-server")
}
