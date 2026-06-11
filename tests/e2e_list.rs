use utils::*;

#[test]
fn shows_main_and_linked_worktrees() {
    let (root, repo) = setup_repo();
    let feature = root.path().join("feature-one");

    assert_success(wt(&repo, &["add", "feature/one"]));

    let list = stdout(wt(&repo, &["list"]));
    assert_worktree_list_contains(&list, "main", &repo);
    assert_worktree_list_contains(&list, "feature/one", &feature);
}
