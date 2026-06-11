use std::fs;
use utils::*;

#[test]
fn shows_main_and_linked_worktrees() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature-one");

    assert_success(wt(&repo, &["add", "feature/one"]));

    let list = stdout(wt(&repo, &["list"]));
    assert!(list.contains(&format!("main\t{}", fs::canonicalize(&repo).unwrap().display())));
    assert!(list.contains(&format!("feature/one\t{}", fs::canonicalize(&feature).unwrap().display())));
}
