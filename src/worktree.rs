use std::{
    env,
    ffi::OsStr,
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use gix::bstr::ByteSlice;
use log::debug;

use crate::{Result, error, fail};

/// Owns native worktree lifecycle operations for one discovered repository.
pub(crate) struct WorktreeManager<'repo> {
    repo: &'repo gix::Repository,
}

impl<'repo> WorktreeManager<'repo> {
    pub(crate) fn new(repo: &'repo gix::Repository) -> Self {
        Self { repo }
    }

    /// Create a branch worktree, failing if the branch is already checked out.
    pub(crate) fn create(&self, branch: &str, dir: Option<&Path>) -> Result<PathBuf> {
        self.create_from_plan(self.create_plan(branch, dir)?)
    }

    /// Return an existing branch worktree or create it from the current HEAD.
    pub(crate) fn resolve_or_create(&self, branch: &str, dir: Option<&Path>) -> Result<ResolvedWorktree> {
        match self.checked_out_path(branch)? {
            Some(path) => Ok(ResolvedWorktree { path, created: false }),
            None => Ok(ResolvedWorktree {
                path: self.create(branch, dir)?,
                created: true,
            }),
        }
    }

    /// Remove a linked branch worktree and its metadata.
    pub(crate) fn remove(&self, branch: &str, force: bool) -> Result<PathBuf> {
        let Some((base, admin_dir)) = self.linked_worktree_by_branch(branch)? else {
            return fail(format!("no linked worktree for branch '{branch}'"));
        };
        debug!(
            "removing worktree for branch '{branch}' at {} with metadata {}",
            base.display(),
            admin_dir.display()
        );

        let worktree = gix::discover(&base)?;
        if !force && has_changes(&worktree)? {
            return fail(format!("worktree for branch '{branch}' has changes; use --force"));
        }

        fs::remove_dir_all(&base)?;
        fs::remove_dir_all(&admin_dir)?;
        Ok(base)
    }

    /// Validate paths and branch checkout data before touching disk.
    fn create_plan(&self, branch: &str, dir: Option<&Path>) -> Result<WorktreeCreatePlan> {
        let main_path = main_worktree_path(self.repo)?;
        debug!("preparing worktree for branch '{branch}'");

        if let Some(path) = self.checked_out_path(branch)? {
            debug!("branch '{branch}' is already checked out at {}", path.display());
            return fail(format!("branch '{branch}' is already checked out at {}", path.display()));
        }

        let worktree_name = worktree_name(branch)?;
        let parent_dir = match dir {
            Some(dir) => absolute_path(dir)?,
            None => main_path
                .parent()
                .ok_or_else(|| error("main worktree has no parent directory"))?
                .to_owned(),
        };
        let target_path = parent_dir.join(&worktree_name);
        let admin_dir = fs::canonicalize(self.repo.common_dir())?.join("worktrees").join(&worktree_name);

        if target_path.exists() {
            return fail(format!("target path already exists: {}", target_path.display()));
        }
        if admin_dir.exists() {
            return fail(format!("worktree metadata already exists: {}", admin_dir.display()));
        }

        let branch_checkout = self.branch_checkout(branch)?;
        Ok(WorktreeCreatePlan {
            branch_ref: branch_ref(branch),
            target_path,
            admin_dir,
            tree_id: branch_checkout.tree_id,
            create_branch_at: branch_checkout.create_branch_at,
        })
    }

    /// Materialize a planned worktree and roll back partial dirs on failure.
    fn create_from_plan(&self, plan: WorktreeCreatePlan) -> Result<PathBuf> {
        debug!(
            "creating worktree files for {} at {} with metadata {}",
            plan.branch_ref,
            plan.target_path.display(),
            plan.admin_dir.display()
        );

        let mut pending = PendingWorktree::default();
        fs::create_dir_all(&plan.target_path)?;
        let target_path = fs::canonicalize(&plan.target_path)?;
        pending.target_path = Some(target_path.clone());

        fs::create_dir_all(&plan.admin_dir)?;
        let admin_dir = fs::canonicalize(&plan.admin_dir)?;
        pending.admin_dir = Some(admin_dir.clone());

        if let Some(commit_id) = plan.create_branch_at {
            self.repo.reference(
                plan.branch_ref.as_str(),
                commit_id,
                gix::refs::transaction::PreviousValue::MustNotExist,
                "wt add",
            )?;
        }

        fs::write(target_path.join(".git"), format!("gitdir: {}\n", admin_dir.display()))?;
        fs::write(admin_dir.join("gitdir"), format!("{}\n", target_path.join(".git").display()))?;
        fs::write(admin_dir.join("commondir"), "../..\n")?;
        fs::write(admin_dir.join("HEAD"), format!("ref: {}\n", plan.branch_ref))?;

        let worktree = gix::discover(&target_path)?;
        let mut index = worktree.index_from_tree(&plan.tree_id)?;
        index.set_path(worktree.index_path());

        let mut options = worktree.checkout_options(gix::worktree::stack::state::attributes::Source::IdMapping)?;
        options.destination_is_initially_empty = true;

        gix::worktree::state::checkout(
            &mut index,
            worktree.workdir().ok_or_else(|| error("created worktree has no workdir"))?,
            worktree.objects.clone().into_arc()?,
            &gix::progress::Discard,
            &gix::progress::Discard,
            &gix::interrupt::IS_INTERRUPTED,
            options,
        )?;
        index.write(Default::default())?;

        pending.commit();
        Ok(target_path)
    }

    /// Resolve the tree to check out and whether a branch ref must be created.
    fn branch_checkout(&self, branch: &str) -> Result<BranchCheckout> {
        let ref_name = branch_ref(branch);
        if let Some(mut reference) = self.repo.try_find_reference(ref_name.as_str())? {
            return Ok(BranchCheckout {
                tree_id: reference.peel_to_commit()?.tree_id()?.detach(),
                create_branch_at: None,
            });
        }

        let commit_id = self.repo.head_id()?.detach();
        let tree_id = self.repo.find_commit(commit_id)?.tree_id()?.detach();
        Ok(BranchCheckout {
            tree_id,
            create_branch_at: Some(commit_id),
        })
    }

    /// Find the path where a branch is already checked out.
    fn checked_out_path(&self, branch: &str) -> Result<Option<PathBuf>> {
        let main_path = main_worktree_path(self.repo)?;
        let main_repo = gix::discover(&main_path)?;
        if branch_name(&main_repo)?.as_deref() == Some(branch) {
            return Ok(Some(main_path));
        }

        Ok(self.linked_worktree_by_branch(branch)?.map(|(base, _)| base))
    }

    /// Search linked worktrees by their checked-out branch name.
    fn linked_worktree_by_branch(&self, branch: &str) -> Result<Option<(PathBuf, PathBuf)>> {
        for proxy in self.repo.worktrees()? {
            let base = proxy.base()?;
            let admin_dir = fs::canonicalize(proxy.git_dir())?;
            let worktree = proxy.into_repo()?;
            if branch_name(&worktree)?.as_deref() == Some(branch) {
                return Ok(Some((base, admin_dir)));
            }
        }

        Ok(None)
    }
}

pub(crate) struct ResolvedWorktree {
    pub(crate) path: PathBuf,
    pub(crate) created: bool,
}

struct WorktreeCreatePlan {
    branch_ref: String,
    target_path: PathBuf,
    admin_dir: PathBuf,
    tree_id: gix::ObjectId,
    create_branch_at: Option<gix::ObjectId>,
}

struct BranchCheckout {
    tree_id: gix::ObjectId,
    create_branch_at: Option<gix::ObjectId>,
}

/// Tracks directories created during worktree creation until checkout succeeds.
#[derive(Default)]
struct PendingWorktree {
    target_path: Option<PathBuf>,
    admin_dir: Option<PathBuf>,
    committed: bool,
}

impl PendingWorktree {
    fn commit(&mut self) {
        self.committed = true;
    }
}

impl Drop for PendingWorktree {
    fn drop(&mut self) {
        if self.committed {
            return;
        }
        if let Some(path) = self.target_path.as_deref() {
            let _ = fs::remove_dir_all(path);
        }
        if let Some(path) = self.admin_dir.as_deref() {
            let _ = fs::remove_dir_all(path);
        }
    }
}

/// Discover a repository from a worktree or its containing worktree root.
pub(crate) fn discover_repo() -> Result<gix::Repository> {
    if let Ok(repo) = gix::discover(".") {
        debug!("discovered git repository from current directory");
        return Ok(repo);
    }

    let cwd = fs::canonicalize(env::current_dir()?)?;
    debug!(
        "current directory is not a worktree; scanning {} for one worktree root",
        cwd.display()
    );
    let mut main_paths = Vec::new();

    for entry in fs::read_dir(&cwd)? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }

        let Ok(repo) = gix::discover(entry.path()) else {
            continue;
        };
        let Ok(main_path) = main_worktree_path(&repo) else {
            continue;
        };
        if main_path.parent() == Some(cwd.as_path()) && !main_paths.contains(&main_path) {
            main_paths.push(main_path);
        }
    }

    match main_paths.as_slice() {
        [main_path] => Ok(gix::discover(main_path)?),
        [] => fail("not in a git repository or worktree root"),
        _ => fail("multiple git worktree roots found; run inside a worktree"),
    }
}

/// Write the main and linked worktrees as tab-separated branch/path rows.
pub(crate) fn write_list(repo: &gix::Repository, mut out: impl Write) -> Result<()> {
    let main_path = main_worktree_path(repo)?;
    let main_repo = gix::discover(&main_path)?;
    debug!("listing worktrees from {}", main_path.display());

    writeln!(
        out,
        "{}\t{}",
        branch_name(&main_repo)?.unwrap_or_else(|| "<detached>".to_owned()),
        main_path.display()
    )?;

    for proxy in repo.worktrees()? {
        let base = proxy.base()?;
        let worktree = proxy.into_repo()?;
        writeln!(
            out,
            "{}\t{}",
            branch_name(&worktree)?.unwrap_or_else(|| "<detached>".to_owned()),
            base.display()
        )?;
    }

    Ok(())
}

fn has_changes(repo: &gix::Repository) -> Result<bool> {
    let mut status = repo.status(gix::progress::Discard)?.into_iter(Vec::<gix::bstr::BString>::new())?;
    Ok(status.next().transpose()?.is_some())
}

fn branch_name(repo: &gix::Repository) -> Result<Option<String>> {
    Ok(repo.head_name()?.map(|name| name.shorten().to_str_lossy().into_owned()))
}

fn branch_ref(branch: &str) -> String {
    format!("refs/heads/{branch}")
}

fn worktree_name(branch: &str) -> Result<String> {
    if branch.is_empty() {
        return fail("branch must not be empty");
    }

    Ok(branch.replace('/', "-"))
}

/// Resolve the main worktree path from the repository common dir.
fn main_worktree_path(repo: &gix::Repository) -> Result<PathBuf> {
    let common_dir = fs::canonicalize(repo.common_dir())?;
    if common_dir.file_name() != Some(OsStr::new(".git")) {
        return fail("bare repositories are not supported");
    }

    let path = common_dir
        .parent()
        .ok_or_else(|| error("main worktree path could not be resolved"))?;
    Ok(fs::canonicalize(path)?)
}

fn absolute_path(path: &Path) -> Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_owned())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}
