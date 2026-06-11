use std::fs;
use utils::*;

#[test]
fn commands_work_from_worktree_root() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    assert_success(wt(temp.path(), &["add", "feature"]));
    assert!(feature.is_dir());

    let list = stdout(wt(temp.path(), &["list"]));
    assert!(list.contains(&format!("main\t{}", fs::canonicalize(&repo).unwrap().display())));
    assert!(list.contains(&format!("feature\t{}", fs::canonicalize(&feature).unwrap().display())));

    let output = stdout(wt(temp.path(), &["switch", "feature"]));
    assert_eq!(output.trim(), fs::canonicalize(&feature).unwrap().display().to_string());

    assert_success(wt(temp.path(), &["remove", "feature"]));
    assert!(!feature.exists());
}
