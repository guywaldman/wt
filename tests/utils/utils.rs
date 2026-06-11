use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

use tempfile::TempDir;

const GIT_REPOSITORY_ENV_VARS: &[&str] = &[
    "GIT_ALTERNATE_OBJECT_DIRECTORIES",
    "GIT_OBJECT_DIRECTORY",
    "GIT_DIR",
    "GIT_WORK_TREE",
    "GIT_IMPLICIT_WORK_TREE",
    "GIT_INDEX_FILE",
    "GIT_PREFIX",
    "GIT_SHALLOW_FILE",
    "GIT_COMMON_DIR",
];

pub fn setup_repo() -> (TempDir, PathBuf) {
    let temp = tempfile::tempdir().unwrap();
    git(temp.path(), &["init", "-q", "--initial-branch", "main", "repo"]);

    let repo = temp.path().join("repo");
    git(&repo, &["config", "user.email", "a@example.com"]);
    git(&repo, &["config", "user.name", "A"]);
    git(&repo, &["config", "core.autocrlf", "false"]);
    fs::write(repo.join("README.md"), "hello\n").unwrap();
    git(&repo, &["add", "README.md"]);
    git(&repo, &["commit", "-q", "-m", "initial"]);

    (temp, repo)
}

pub fn assert_init_wrapper_switches_cwd(shell: &str, script: &str) {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");
    let wt_bin = wt_bin();
    let bin_dir = wt_bin.parent().unwrap();
    let old_path = env::var_os("PATH").unwrap_or_default();
    let path = env::join_paths([bin_dir.to_owned()].into_iter().chain(env::split_paths(&old_path))).unwrap();

    let output = Command::new(shell)
        .arg("-c")
        .arg(script)
        .current_dir(&repo)
        .env("PATH", path)
        .env("WT_BIN", wt_bin)
        .output()
        .unwrap();

    assert_eq!(stdout(output).trim(), fs::canonicalize(feature).unwrap().display().to_string());
}

pub fn command_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok()
}

pub fn wt(repo: &Path, args: &[&str]) -> Output {
    Command::new(wt_bin()).current_dir(repo).args(args).output().unwrap()
}

pub fn wt_with_env(repo: &Path, args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut command = Command::new(wt_bin());
    command.current_dir(repo).args(args).envs(envs.iter().copied());
    command.output().unwrap()
}

pub fn git(cwd: &Path, args: &[&str]) {
    assert_success(git_output(cwd, args));
}

pub fn git_output(cwd: &Path, args: &[&str]) -> Output {
    git_command(cwd, args).output().unwrap()
}

pub fn assert_success(output: Output) {
    assert!(
        output.status.success(),
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn stdout(output: Output) -> String {
    assert_success_status(&output);
    String::from_utf8(output.stdout).unwrap()
}

pub fn stderr(output: Output) -> String {
    String::from_utf8(output.stderr).unwrap()
}

fn wt_bin() -> PathBuf {
    if let Some(path) = env::var_os("CARGO_BIN_EXE_wt") {
        return PathBuf::from(path);
    }
    if let Some(path) = option_env!("CARGO_BIN_EXE_wt") {
        return PathBuf::from(path);
    }

    let mut path = env::current_exe().unwrap();
    path.pop();
    if path.file_name().is_some_and(|name| name == "deps") {
        path.pop();
    }
    path.push(format!("wt{}", env::consts::EXE_SUFFIX));
    path
}

fn git_command(cwd: &Path, args: &[&str]) -> Command {
    let mut command = Command::new("git");
    clear_git_env(&mut command);
    command.current_dir(cwd).args(args);
    command
}

fn clear_git_env(command: &mut Command) {
    for key in GIT_REPOSITORY_ENV_VARS {
        command.env_remove(key);
    }
}

fn assert_success_status(output: &Output) {
    assert!(
        output.status.success(),
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
