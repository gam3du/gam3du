use crate::{
    ace,
    util::{check_all_programs, Program},
};
use anyhow::Context;
use pico_args::Arguments;
use std::path::Path;
use xshell::Shell;

pub(crate) fn run(shell: &Shell, mut args: Arguments) -> anyhow::Result<()> {
    let no_serve = args.contains("--no-serve");
    let release = args.contains("--release");

    let programs_needed: &[_] = if no_serve {
        &[Program {
            crate_name: "wasm-bindgen-cli",
            binary_name: "wasm-bindgen",
        }]
    } else {
        &[
            Program {
                crate_name: "wasm-bindgen-cli",
                binary_name: "wasm-bindgen",
            },
            Program {
                crate_name: "simple-http-server",
                binary_name: "simple-http-server",
            },
        ]
    };

    check_all_programs(programs_needed)?;

    let release_flag: &[_] = if release { &["--release"] } else { &[] };
    let output_dir = if release { "release" } else { "debug" };

    log::info!("building robot application");

    let cargo_args = args.finish();

    xshell::cmd!(
        shell,
        "cargo build --target wasm32-unknown-unknown --package application-robot {release_flag...}"
    )
    .args(&cargo_args)
    .quiet()
    .run()
    .context("Failed to build webgl examples for wasm")?;

    log::info!("running wasm-bindgen on webgl examples");

    xshell::cmd!(
        shell,
        "wasm-bindgen target/wasm32-unknown-unknown/{output_dir}/application-robot.wasm --target web --no-typescript --out-dir target/generated --out-name application-robot"
    )
    .quiet()
    .run()
    .context("Failed to run wasm-bindgen")?;

    let static_files = shell
        .read_dir("static")
        .context("Failed to enumerate static files")?;

    for file in static_files {
        log::info!("copying static file \"{}\"", file.canonicalize()?.display());

        shell
            .copy_file(&file, "target/generated")
            .with_context(|| format!("Failed to copy static file \"{}\"", file.display()))?;
    }

    ace::download(shell, Path::new("target/generated"))?;

    if !no_serve {
        log::info!("serving on port 8000");

        // Explicitly specify the IP address to 127.0.0.1 since otherwise simple-http-server will
        // print http://0.0.0.0:8000 as url which is not a secure context and thus doesn't allow
        // running WebGPU!
        xshell::cmd!(
            shell,
            "simple-http-server target/generated -c wasm,html,js -i --coep --coop --ip 127.0.0.1 --nocache"
        )
        .quiet()
        .run()
        .context("Failed to simple-http-server")?;
    }

    Ok(())
}
