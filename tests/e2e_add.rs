use utils::*;

#[test]
fn creates_branch_worktree() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature-one");

    assert_success(wt(&repo, &["add", "feature/one"]));

    assert!(feature.is_dir());
    let branch = stdout(git_output(&feature, &["branch", "--show-current"]));
    assert_eq!(branch.trim(), "feature/one");
}

#[test]
fn accepts_parent_dir() {
    let (temp, repo) = setup_repo();
    let parent = temp.path().join("custom");
    let path = parent.join("feature");

    assert_success(wt(&repo, &["add", "feature", parent.to_str().unwrap()]));

    assert!(path.is_dir());
}

#[test]
fn duplicate_fails() {
    let (_temp, repo) = setup_repo();

    assert_success(wt(&repo, &["add", "feature"]));
    let output = wt(&repo, &["add", "feature"]);

    assert!(!output.status.success());
    assert!(stderr(output).contains("already checked out"));
}

#[test]
fn derived_path_collision_does_not_create_branch() {
    let (_temp, repo) = setup_repo();

    assert_success(wt(&repo, &["add", "feature-foo"]));
    let output = wt(&repo, &["add", "feature/foo"]);

    assert!(!output.status.success());
    assert!(stderr(output).contains("target path already exists"));
    assert!(
        !git_output(&repo, &["show-ref", "--verify", "--quiet", "refs/heads/feature/foo"])
            .status
            .success()
    );
}
