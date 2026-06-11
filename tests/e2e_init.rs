use utils::*;

#[test]
fn emits_completion_and_cwd_switching_wrapper() {
    let (_root, repo) = setup_repo();

    let init = stdout(wt(&repo, &["init", "zsh"]));

    assert!(init.contains("#compdef wt"));
    assert!(init.contains("wt() {"));
    assert!(init.contains("command wt switch"));
    assert!(init.contains("cd \"$target\""));
}

#[test]
fn bash_wrapper_switches_cwd_inside_child_shell() {
    if cfg!(windows) || !command_exists("bash") {
        return;
    }

    assert_init_wrapper_switches_cwd(
        "bash",
        "set -euo pipefail\neval \"$(\"$WT_BIN\" init bash)\"\nwt switch feature\npwd\n",
    );
}

#[test]
fn zsh_wrapper_switches_cwd_inside_child_shell() {
    if cfg!(windows) || !command_exists("zsh") {
        return;
    }

    assert_init_wrapper_switches_cwd(
        "zsh",
        "set -e\nautoload -Uz compinit\ncompinit -D\neval \"$(\"$WT_BIN\" init zsh)\"\nwt switch feature\npwd\n",
    );
}
