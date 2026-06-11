use utils::*;

#[test]
fn prints_existing_worktree_path() {
    let (root, repo) = setup_repo();
    let feature = root.path().join("feature");

    assert_success(wt(&repo, &["add", "feature"]));

    let output = stdout(wt(&repo, &["switch", "feature"]));
    assert_path_eq(output.trim(), &feature);

    let output = stdout(wt(&feature, &["switch", "main"]));
    assert_path_eq(output.trim(), &repo);
}

#[test]
fn creates_missing_worktree() {
    let (root, repo) = setup_repo();
    let feature = root.path().join("feature");

    let output = stdout(wt(&repo, &["switch", "feature"]));

    assert_path_eq(output.trim(), &feature);
    assert_eq!(stdout(git_output(&feature, &["branch", "--show-current"])).trim(), "feature");
}

#[test]
fn accepts_target_path() {
    let (root, repo) = setup_repo();
    let path = root.path().join("custom-feature");

    let output = stdout(wt(&repo, &["switch", "feature", path.to_str().unwrap()]));

    assert_path_eq(output.trim(), &path);
}
