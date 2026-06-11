use utils::*;

#[test]
fn creates_branch_worktree() {
    let (root, repo) = setup_repo();
    let feature = root.path().join("feature-one");

    assert_success(wt(&repo, &["add", "feature/one"]));

    assert!(feature.is_dir());
    let branch = stdout(git_output(&feature, &["branch", "--show-current"]));
    assert_eq!(branch.trim(), "feature/one");
}

#[test]
fn accepts_target_path() {
    let (root, repo) = setup_repo();
    let path = root.path().join("custom-feature");

    assert_success(wt(&repo, &["add", "feature", path.to_str().unwrap()]));

    assert!(path.is_dir());
}

#[test]
fn accepts_slashy_branch_with_explicit_target_path() {
    let (root, _repo) = setup_repo();
    let path = root.path().join("my-feature");

    assert_success(wt(root.path(), &["add", "user/foo/my-feature", "my-feature"]));

    assert!(path.is_dir());
    assert!(!path.join("user-foo-my-feature").exists());
    let branch = stdout(git_output(&path, &["branch", "--show-current"]));
    assert_eq!(branch.trim(), "user/foo/my-feature");
}

#[test]
fn resolves_relative_target_path_from_linked_worktree_root() {
    let (root, repo) = setup_repo();
    let feature = root.path().join("feature");
    let path = root.path().join("my-feature");

    assert_success(wt(&repo, &["add", "feature"]));
    assert_success(wt(&feature, &["add", "user/foo/my-feature", "my-feature"]));

    assert!(path.is_dir());
    assert!(!feature.join("my-feature").exists());
    let branch = stdout(git_output(&path, &["branch", "--show-current"]));
    assert_eq!(branch.trim(), "user/foo/my-feature");
}

#[test]
fn duplicate_fails() {
    let (_root, repo) = setup_repo();

    assert_success(wt(&repo, &["add", "feature"]));
    let output = wt(&repo, &["add", "feature"]);

    assert!(!output.status.success());
    assert!(stderr(output).contains("already checked out"));
}

#[test]
fn derived_path_collision_does_not_create_branch() {
    let (_root, repo) = setup_repo();

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
