use anyhow::Result;
use std::{
    path::{Path, PathBuf},
    process::Command,
};

pub fn jj_command<'c>(command: &'c mut Command, repo: &Path) -> &'c mut Command {
    command.current_dir(repo).env("JJ_CONFIG", "/dev/null")
}

pub fn log(repo: &Path, revisions: &str, template: &str) -> Result<String> {
    let output = jj_command(&mut Command::new("jj"), repo)
        .args(&["log", "--no-graph", "-r", revisions, "-T", template])
        .output()?;
    anyhow::ensure!(output.status.success(), String::from_utf8(output.stderr)?);

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

pub fn show(repo: &Path, commit_id: &str) -> Result<String> {
    let output = jj_command(&mut Command::new("jj"), repo)
        .args(&["show", commit_id, "--config", "ui.diff.format=git", "-s"])
        .output()?;
    anyhow::ensure!(output.status.success(), String::from_utf8(output.stderr)?);

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

pub fn diff(repo: &Path, path: &Path) -> Result<String> {
    let output = jj_command(&mut Command::new("jj"), repo)
        .arg("diff")
        .arg(path)
        .args(&["--config", "ui.diff.format=git"])
        .output()?;
    anyhow::ensure!(output.status.success(), String::from_utf8(output.stderr)?);

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout)
}

pub fn workspace_root(repo: &Path) -> Result<PathBuf> {
    let output = jj_command(&mut Command::new("jj"), repo)
        .args(&["workspace", "root"])
        .output()?;
    anyhow::ensure!(output.status.success(), String::from_utf8(output.stderr)?);

    let stdout = String::from_utf8(output.stdout)?;
    Ok(stdout.trim().to_owned().into())
}
