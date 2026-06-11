use std::fs;
use utils::*;

#[test]
fn copies_unstaged_changes_to_target_worktree() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    fs::write(repo.join("README.md"), "hello\nsource change\n").unwrap();

    let output = stdout(wt(&repo, &["fork", "feature"]));
    assert_path_eq(output.trim(), &feature);
    assert_eq!(read_text(feature.join("README.md")), "hello\nsource change\n");
    assert_eq!(stdout(git_output(&repo, &["status", "--short"])), " M README.md\n");
    assert_eq!(stdout(git_output(&feature, &["status", "--short"])), " M README.md\n");
}

#[test]
fn copies_staged_changes_as_staged() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    fs::write(repo.join("README.md"), "hello\nstaged change\n").unwrap();
    git(&repo, &["add", "README.md"]);

    assert_success(wt(&repo, &["fork", "feature", "--staged"]));

    assert_eq!(stdout(git_output(&feature, &["status", "--short"])), "M  README.md\n");
    assert_eq!(read_text(feature.join("README.md")), "hello\nstaged change\n");
    assert_eq!(stdout(git_output(&repo, &["status", "--short"])), "M  README.md\n");
}

#[test]
fn respects_pathspecs_after_double_dash() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    fs::write(repo.join("OTHER.md"), "other\n").unwrap();
    git(&repo, &["add", "OTHER.md"]);
    git(&repo, &["commit", "-q", "-m", "other"]);

    fs::write(repo.join("README.md"), "hello\nreadme change\n").unwrap();
    fs::write(repo.join("OTHER.md"), "other\nother change\n").unwrap();

    assert_success(wt(&repo, &["fork", "feature", "--", "README.md"]));

    assert_eq!(read_text(feature.join("README.md")), "hello\nreadme change\n");
    assert_eq!(read_text(feature.join("OTHER.md")), "other\n");
}

#[test]
fn ignores_inherited_git_hook_env() {
    let (temp, repo) = setup_repo();
    let feature = temp.path().join("feature");

    fs::write(repo.join("README.md"), "hello\nsource change\n").unwrap();

    assert_success(wt_with_env(
        &repo,
        &["fork", "feature"],
        &[("GIT_DIR", ".git"), ("GIT_INDEX_FILE", ".git/index")],
    ));

    assert_eq!(read_text(feature.join("README.md")), "hello\nsource change\n");
    assert_eq!(stdout(git_output(&feature, &["status", "--short"])), " M README.md\n");
}
