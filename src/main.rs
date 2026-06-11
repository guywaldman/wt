use std::{
    error::Error,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use clap::{CommandFactory, Parser, Subcommand, ValueHint};
use log::{debug, info};

mod init;
mod patch;
mod worktree;

use init::InitShell;
use worktree::{WorktreeManager, discover_repo};

pub(crate) type Result<T> = std::result::Result<T, Box<dyn Error>>;

const AFTER_HELP: &str = "\
Examples:
  wt init zsh
  wt list
  wt add feature/auth
  wt fork feature/auth --staged -- src/main.rs
  wt remove --force feature/auth

Shell integration:
  eval \"$(wt init zsh)\"

Logging:
  Set RUST_LOG=wt=debug for diagnostics.";

fn main() {
    pretty_env_logger::init();

    if let Err(err) = run(Cli::parse(), io::stdout()) {
        eprintln!("wt: {err}");
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(
    name = "wt",
    version,
    about = "Ergonomic git worktree helper CLI",
    long_about = "Lightweight git worktree helper CLI for working with Git worktrees (listing, creating, switching, removing, and more).",
    arg_required_else_help = true,
    after_help = AFTER_HELP
)]
struct Cli {
    /// Command to run.
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Print shell setup code for completions and cwd switching.
    Init {
        /// Shell to initialize.
        #[arg(value_enum, value_name = "SHELL")]
        shell: InitShell,
    },
    /// Print all worktrees as `<branch><tab><path>`.
    List,
    /// Resolve BRANCH's worktree path for shell-driven cwd switching.
    /// This is an internal command, used within the shell wrapper emitted by `init`.
    #[command(hide = true)]
    Switch {
        /// Branch to switch to.
        #[arg(value_name = "BRANCH")]
        branch: String,
        /// Path for a newly created worktree.
        ///
        /// Relative paths resolve from the worktree root. Defaults to a sibling of the main worktree.
        #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
    },
    /// Create a worktree for BRANCH.
    Add {
        /// Branch to create or check out.
        #[arg(value_name = "BRANCH")]
        branch: String,
        /// Path for the new worktree.
        ///
        /// Relative paths resolve from the worktree root. Defaults to a sibling of the main worktree.
        #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
    },
    /// Remove the linked worktree for BRANCH.
    Remove {
        /// Remove even if the worktree has local changes.
        #[arg(long)]
        force: bool,
        /// Branch whose linked worktree should be removed.
        #[arg(value_name = "BRANCH")]
        branch: String,
    },
    /// Copy current changes into BRANCH's worktree, creating it first if needed.
    Fork {
        /// Target branch for the forked changes.
        #[arg(value_name = "BRANCH")]
        branch: String,
        /// Path for a newly created target worktree.
        ///
        /// Relative paths resolve from the worktree root. Defaults to a sibling of the main worktree.
        #[arg(value_name = "PATH", value_hint = ValueHint::DirPath)]
        path: Option<PathBuf>,
        /// Copy staged changes instead of unstaged changes.
        #[arg(long)]
        staged: bool,
        /// Optional pathspecs passed to `git diff`.
        ///
        /// Use `--` before paths to separate them from wt options.
        #[arg(last = true, value_name = "PATH", value_hint = ValueHint::AnyPath)]
        paths: Vec<PathBuf>,
    },
}

/// Dispatch parsed CLI commands to their command handlers.
fn run(cli: Cli, out: impl Write) -> Result<()> {
    match cli.command {
        Commands::Init { shell } => init::write(shell, Cli::command(), out),
        Commands::List => list(out),
        Commands::Switch { branch, path } => switch(&branch, path.as_deref(), out),
        Commands::Add { branch, path } => add(&branch, path.as_deref()),
        Commands::Remove { force, branch } => remove(&branch, force),
        Commands::Fork {
            branch,
            path,
            staged,
            paths,
        } => fork(&branch, path.as_deref(), staged, &paths, out),
    }
}

/// Print every worktree as `<branch><tab><path>`.
fn list(mut out: impl Write) -> Result<()> {
    let repo = discover_repo()?;
    worktree::write_list(&repo, &mut out)
}

/// Create a linked worktree for a branch.
fn add(branch: &str, path: Option<&Path>) -> Result<()> {
    let repo = discover_repo()?;
    let path = WorktreeManager::new(&repo).create(branch, path)?;
    info!("created worktree for branch '{branch}' at {}", path.display());
    Ok(())
}

/// Print an existing branch worktree path, creating it first if needed.
fn switch(branch: &str, path: Option<&Path>, mut out: impl Write) -> Result<()> {
    let repo = discover_repo()?;
    let target = WorktreeManager::new(&repo).resolve_or_create(branch, path)?;
    if target.created {
        info!("created switch target for branch '{branch}' at {}", target.path.display());
    } else {
        debug!("switch target branch '{branch}' already exists at {}", target.path.display());
    }

    writeln!(out, "{}", target.path.display())?;
    Ok(())
}

/// Copy selected local changes into a target branch worktree.
fn fork(branch: &str, path: Option<&Path>, staged: bool, paths: &[PathBuf], mut out: impl Write) -> Result<()> {
    let repo = discover_repo()?;
    let source_path = fs::canonicalize(repo.workdir().ok_or_else(|| error("fork requires a worktree"))?)?;
    debug!(
        "capturing {} changes from {}",
        if staged { "staged" } else { "unstaged" },
        source_path.display()
    );
    let diff = patch::diff(&source_path, staged, paths)?;
    if diff.is_empty() {
        return fail("no changes to fork");
    }
    debug!("captured {} bytes of diff", diff.len());

    let target = WorktreeManager::new(&repo).resolve_or_create(branch, path)?;
    if target.created {
        info!("created fork target for branch '{branch}' at {}", target.path.display());
    } else {
        debug!("fork target branch '{branch}' already exists at {}", target.path.display());
    }
    let target_path = fs::canonicalize(target.path)?;
    if target_path == source_path {
        return fail("cannot fork changes to the current worktree");
    }

    patch::apply(&target_path, staged, &diff)?;
    info!(
        "forked {} changes from {} to {}",
        if staged { "staged" } else { "unstaged" },
        source_path.display(),
        target_path.display()
    );
    writeln!(out, "{}", target_path.display())?;
    Ok(())
}

/// Remove a linked worktree after enforcing dirty-worktree safety.
fn remove(branch: &str, force: bool) -> Result<()> {
    let repo = discover_repo()?;
    let path = WorktreeManager::new(&repo).remove(branch, force)?;
    info!("removed worktree for branch '{branch}' at {}", path.display());
    Ok(())
}

pub(crate) fn fail<T>(message: impl Into<String>) -> Result<T> {
    Err(error(message).into())
}

pub(crate) fn error(message: impl Into<String>) -> io::Error {
    io::Error::other(message.into())
}
