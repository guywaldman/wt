use utils::*;

#[test]
fn commands_work_from_worktree_root() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    assert_success(wt(temp.path(), &["add", "feature"]));
    assert!(feature.is_dir());

    let list = stdout(wt(temp.path(), &["list"]));
    assert_worktree_list_contains(&list, "main", &repo);
    assert_worktree_list_contains(&list, "feature", &feature);

    let output = stdout(wt(temp.path(), &["switch", "feature"]));
    assert_path_eq(output.trim(), &feature);

    assert_success(wt(temp.path(), &["remove", "feature"]));
    assert!(!feature.exists());
}
