use utils::*;

#[test]
fn commands_work_from_worktree_root() {
    let (root, repo) = setup_repo();
    let feature = root.path().join("feature");

    assert_success(wt(root.path(), &["add", "feature"]));
    assert!(feature.is_dir());

    let list = stdout(wt(root.path(), &["list"]));
    assert_worktree_list_contains(&list, "main", &repo);
    assert_worktree_list_contains(&list, "feature", &feature);

    let output = stdout(wt(root.path(), &["switch", "feature"]));
    assert_path_eq(output.trim(), &feature);

    assert_success(wt(root.path(), &["remove", "feature"]));
    assert!(!feature.exists());
}
