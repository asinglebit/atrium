use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use git2::{BranchType, Error, Repository, WorktreeAddOptions};

/// Worktrees belong to the repository that owns them, which is not the one you
/// are standing in when you are standing in a worktree.
fn owner(repo: &Repository) -> Result<Repository, Error> {
    Repository::open(repo.commondir())
}

/// A name becomes both a directory and a branch, so anything that would change
/// what either of those means is refused rather than sanitised.
pub fn is_valid_name(name: &str) -> bool {
    !name.is_empty() && name != "." && name != ".." && !name.contains('/') && !name.contains('\\')
}

/// Beside the repository rather than inside it, named for both -- guitar's
/// convention, and the one the worktrees already on this machine follow.
pub fn default_path(repo_root: &Path, name: &str) -> PathBuf {
    let parent = repo_root.parent().unwrap_or_else(|| Path::new("."));
    let repo_name = repo_root.file_name().and_then(|name| name.to_str()).unwrap_or("worktree");
    parent.join(format!("{repo_name}-{name}"))
}

/// The working directory of the repository that owns `repo`, which is where
/// `default_path` measures from.
pub fn root_of(repo: &Repository) -> Result<PathBuf, Error> {
    let owner = owner(repo)?;
    owner.workdir().map(Path::to_path_buf).ok_or_else(|| Error::from_str("a bare repository has no worktree to sit beside"))
}

/// Make one, on a new branch of the same name, based on where HEAD is now.
pub fn create(repo: &Repository, name: &str, path: &Path) -> Result<PathBuf, Error> {
    if !is_valid_name(name) {
        return Err(Error::from_str("Worktree names cannot be empty or contain path separators"));
    }
    let repo = owner(repo)?;
    let head = repo.head()?.peel_to_commit()?;

    let result = {
        let branch = repo.branch(name, &head, false)?;
        let reference = branch.into_reference();
        let mut options = WorktreeAddOptions::new();
        options.reference(Some(&reference));
        repo.worktree(name, path, Some(&options)).map(|_| ())
    };

    // A branch left behind by a worktree that failed to appear is the failure
    // worth undoing: the next attempt at the same name would collide with it.
    if let Err(error) = result {
        if let Ok(mut branch) = repo.find_branch(name, BranchType::Local) {
            let _ = branch.delete();
        }
        return Err(error);
    }

    Ok(path.to_path_buf())
}

/// Every worktree the repository owns, by the path it stands in. The main
/// checkout is not one of them.
pub fn list(repo: &Repository) -> Result<Vec<PathBuf>, Error> {
    let repo = owner(repo)?;
    let mut paths: Vec<PathBuf> = repo.worktrees()?.iter().flatten().filter_map(|name| repo.find_worktree(name).ok()).map(|worktree| worktree.path().to_path_buf()).collect();
    paths.sort();
    Ok(paths)
}

/// What worktrees each repository had the last time atrium looked, so one made
/// by something that never said so can still be noticed.
#[derive(Default)]
pub struct Watch {
    seen: HashMap<PathBuf, Vec<PathBuf>>,
}

impl Watch {
    pub fn new() -> Self {
        Self::default()
    }

    /// The worktrees that have appeared since the last look. A repository seen
    /// for the first time reports nothing: its worktrees were already there,
    /// and announcing every one of them at startup is noise rather than news.
    pub fn tick(&mut self, repos: &[PathBuf]) -> Vec<PathBuf> {
        let mut appeared = Vec::new();
        for path in repos {
            let Ok(repo) = Repository::discover(path) else { continue };
            let Ok(now) = list(&repo) else { continue };
            // Keyed on the owner, so an agent in a worktree and one in the main
            // checkout are looking at the same set rather than two of them.
            let key = repo.commondir().to_path_buf();
            if let Some(before) = self.seen.get(&key) {
                appeared.extend(now.iter().filter(|path| !before.contains(path)).cloned());
            }
            self.seen.insert(key, now);
        }
        appeared.sort();
        appeared.dedup();
        appeared
    }
}

#[cfg(test)]
#[path = "../tests/core/worktree.rs"]
mod tests;
