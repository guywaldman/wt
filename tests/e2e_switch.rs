use std::fs;
use utils::*;

#[test]
fn prints_existing_worktree_path() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    assert_success(wt(&repo, &["add", "feature"]));

    let output = stdout(wt(&repo, &["switch", "feature"]));
    assert_eq!(output.trim(), fs::canonicalize(&feature).unwrap().display().to_string());

    let output = stdout(wt(&feature, &["switch", "main"]));
    assert_eq!(output.trim(), fs::canonicalize(&repo).unwrap().display().to_string());
}

#[test]
fn creates_missing_worktree() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    let output = stdout(wt(&repo, &["switch", "feature"]));

    assert_eq!(output.trim(), fs::canonicalize(&feature).unwrap().display().to_string());
    assert_eq!(stdout(git_output(&feature, &["branch", "--show-current"])).trim(), "feature");
}

#[test]
fn accepts_parent_dir() {
    let (temp, repo) = setup_repo();
    let parent = temp.path().join("custom");
    let path = parent.join("feature");

    let output = stdout(wt(&repo, &["switch", "feature", parent.to_str().unwrap()]));

    assert_eq!(output.trim(), fs::canonicalize(&path).unwrap().display().to_string());
}
