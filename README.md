# wt

Small git worktree helper CLI, focused on ergonomics.

## Installation

```sh
# Homebrew
brew install guywaldman/tap/wt

# From source
cargo install --git https://github.com/guywaldman/wt --locked
```

## Shell setup

```sh
# zsh
echo 'eval "$(wt init zsh)"' >> ~/.zshrc

# bash
echo 'eval "$(wt init bash)"' >> ~/.bashrc

# fish
echo 'wt init fish | source' >> ~/.config/fish/config.fish
```

This enables completions and makes `wt switch` change the current shell's cwd.

## Commands

```sh
# Lists all worktrees as `<branch><tab><path>`.
wt list

# Creates a worktree for `branch`. If the branch does not exist, it is created from the current `HEAD`.
wt add <branch> [dir]

# Switches cwd to `branch`'s worktree. Creates it first if it does not exist.
wt switch <branch> [dir]

# Copies current changes into `branch`'s worktree. Copies unstaged changes by default; `--staged` copies staged changes and stages them in the target worktree.
wt fork <branch> [dir] [--staged] [-- <paths>...]

# Removes the linked worktree for `branch`. Refuses dirty worktrees unless `--force` is passed.
wt remove [--force] <branch>
```
