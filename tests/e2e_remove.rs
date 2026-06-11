use std::fs;
use utils::*;

#[test]
fn deletes_clean_worktree_and_metadata() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");
    let metadata = repo.join(".git/worktrees/feature");

    assert_success(wt(&repo, &["add", "feature"]));
    assert_success(wt(&repo, &["remove", "feature"]));

    assert!(!feature.exists());
    assert!(!metadata.exists());
}

#[test]
fn requires_force_for_dirty_worktree() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    assert_success(wt(&repo, &["add", "feature"]));
    fs::write(feature.join("scratch.txt"), "dirty\n").unwrap();

    let output = wt(&repo, &["remove", "feature"]);
    assert!(!output.status.success());
    assert!(feature.exists());
    assert!(stderr(output).contains("use --force"));

    assert_success(wt(&repo, &["remove", "--force", "feature"]));
    assert!(!feature.exists());
}
