use std::{io, path::Path, process::Command};

use anyhow::Context;
use xshell::Shell;

pub(crate) struct Program {
    pub crate_name: &'static str,
    pub binary_name: &'static str,
}

pub(crate) fn check_all_programs(programs: &[Program]) -> anyhow::Result<()> {
    let mut failed_crates = Vec::new();
    for &Program {
        crate_name,
        binary_name,
    } in programs
    {
        let mut cmd = Command::new(binary_name);
        cmd.arg("--help");
        let output = cmd.output();
        match output {
            Ok(_output) => {
                log::info!("Checking for {binary_name} in PATH: ✅");
            }
            Err(error) if matches!(error.kind(), io::ErrorKind::NotFound) => {
                log::error!("Checking for {binary_name} in PATH: ❌");
                failed_crates.push(crate_name);
            }
            Err(error) => {
                log::error!("Checking for {binary_name} in PATH: ❌");
                anyhow::bail!("Unknown IO error: {error}");
            }
        }
    }

    if !failed_crates.is_empty() {
        log::error!(
            "Please install them with: cargo install {}",
            failed_crates.join(" ")
        );

        anyhow::bail!("Missing required programs");
    }

    Ok(())
}

pub(crate) fn copy(
    shell: &Shell,
    source_path: &Path,
    destination_path: &Path,
) -> anyhow::Result<()> {
    if source_path.is_file() {
        log::info!(
            "copying file \"{}\" → \"{}\"",
            source_path.canonicalize()?.display(),
            destination_path.display()
        );

        shell
            .copy_file(source_path, destination_path)
            .with_context(|| {
                format!(
                    "Failed to copy file \"{}\" → \"{}\"",
                    source_path.display(),
                    destination_path.display()
                )
            })?;
    } else if source_path.is_dir() {
        let static_files = shell.read_dir(source_path).context(format!(
            "Failed to enumerate files in {}",
            source_path.display()
        ))?;

        let destination_path = &destination_path.join(
            source_path
                .file_name()
                .context(format!("invalid source path: {}", source_path.display()))?,
        );
        shell.create_dir(destination_path).context(format!(
            "failed to create destination path: {}",
            destination_path.display(),
        ))?;

        for file in static_files {
            copy(shell, &file, destination_path)?;
        }
    } else {
        todo!();
    }

    Ok(())
}

pub(crate) fn copy_content(
    shell: &Shell,
    source_path: &Path,
    destination_path: &Path,
) -> anyhow::Result<()> {
    shell.create_dir(destination_path).context(format!(
        "failed to create destination path: {}",
        destination_path.display(),
    ))?;

    if source_path.is_dir() {
        let static_files = shell.read_dir(source_path).context(format!(
            "Failed to enumerate files in {}",
            source_path.display()
        ))?;

        for file in static_files {
            copy(shell, &file, destination_path)?;
        }
    } else {
        todo!("{source_path:?}");
    }

    Ok(())
}
