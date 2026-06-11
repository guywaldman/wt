use std::{
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use log::debug;

use crate::{Result, error, fail};

/// Capture a binary diff from the source worktree.
pub(crate) fn diff(workdir: &Path, staged: bool, paths: &[PathBuf]) -> Result<Vec<u8>> {
    let mut args = vec!["diff", "--binary"];
    if staged {
        args.push("--cached");
    }

    git_output(workdir, &args, paths)
}

/// Apply a captured binary diff to the target worktree.
pub(crate) fn apply(workdir: &Path, staged: bool, input: &[u8]) -> Result<()> {
    let mut command = Command::new("git");
    command
        .current_dir(workdir)
        .arg("apply")
        .arg("--binary")
        .stdin(Stdio::piped())
        .stderr(Stdio::piped());
    if staged {
        command.arg("--index");
    }
    debug!("running git apply in {}", workdir.display());

    let mut child = command.spawn()?;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| error("failed to open git apply stdin"))?
        .write_all(input)?;

    let output = child.wait_with_output()?;
    if !output.status.success() {
        return fail(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    Ok(())
}

fn git_output(workdir: &Path, args: &[&str], paths: &[PathBuf]) -> Result<Vec<u8>> {
    let mut command = Command::new("git");
    command.current_dir(workdir).args(args);
    if !paths.is_empty() {
        command.arg("--").args(paths);
    }
    debug!("running git {} in {}", args.join(" "), workdir.display());

    let output = command.output()?;
    if !output.status.success() {
        return fail(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    Ok(output.stdout)
}
