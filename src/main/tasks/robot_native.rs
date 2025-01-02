use anyhow::Context;
use pico_args::Arguments;
use xshell::Shell;

const PACKAGE_NAME: &str = "application-robot-native";

pub(crate) fn run(shell: &Shell, mut args: Arguments) -> anyhow::Result<()> {
    let is_release = args.contains("--release");
    let release_flag: &[_] = if is_release { &["--release"] } else { &[] };

    let cargo_args = args.finish();

    shell.change_dir("workspace-common");

    xshell::cmd!(
        shell,
        "cargo run --jobs -1 --package {PACKAGE_NAME} {release_flag...}"
    )
    .args(cargo_args)
    // .quiet()
    .run()
    .context(format!("Failed to build {PACKAGE_NAME}"))?;

    // back to project root
    shell.change_dir("..");

    Ok(())
}
