use std::{fs, io::Write, path::Path};

use anyhow::Context;
use xshell::Shell;

pub(crate) fn download(shell: &Shell, target_dir: &Path) -> anyhow::Result<()> {
    log::info!("downloading ACE editor");

    let files = [
        (
            "https://raw.githubusercontent.com/ajaxorg/ace-builds/refs/heads/master/src-noconflict/ace.js",
            target_dir.join("ace.js"),
        ),
        (
            "https://raw.githubusercontent.com/ajaxorg/ace-builds/refs/heads/master/src-noconflict/theme-monokai.js",
            target_dir.join("theme-monokai.js"),
        ),
        (
            "https://raw.githubusercontent.com/ajaxorg/ace-builds/refs/heads/master/src-noconflict/mode-python.js",
            target_dir.join("mode-python.js"),
        ),
    ];

    shell.create_dir(target_dir).context(format!(
        "failed to create destination path: {}",
        target_dir.display(),
    ))?;

    for (url, ref target) in files {
        let target_str = target.display();
        if fs::exists(target).context(format!("failed to test {target_str} for existence"))? {
            log::info!("skipping download for existing file: {target_str}");
            continue;
        }
        log::info!("downloading {url} → {target_str}");

        let resp = reqwest::blocking::get(url).context("request failed")?;
        let body = resp.bytes().context("body invalid")?;
        let mut out = fs::File::create(target).context("failed to create file")?;
        out.write_all(&body).context("failed to copy content")?;
        out.flush().context("failed to copy content")?;
    }

    Ok(())
}
