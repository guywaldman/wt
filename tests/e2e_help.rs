use utils::*;

#[test]
fn shows_init_and_hides_internal_switch() {
    let (_root, repo) = setup_repo();

    let help = stdout(wt(&repo, &["--help"]));

    assert!(help.contains("  init"));
    assert!(!help.contains("  switch"));
}
