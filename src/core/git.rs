use std::path::Path;

use git2::{Repository, StatusOptions};

/// What a sidebar row says about the repository an agent is working in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GitContext {
    pub branch: String,
    pub dirty: bool,
}

/// None when the path is not in a repository at all.
pub fn context_for(path: &Path) -> Option<GitContext> {
    let repo = Repository::discover(path).ok()?;
    let branch = head_name(&repo);
    let dirty = is_dirty(&repo);
    Some(GitContext { branch, dirty })
}

/// A branch by name, a detached head by short hash, and a repo with no commits
/// yet says so rather than looking broken.
fn head_name(repo: &Repository) -> String {
    match repo.head() {
        Ok(head) if head.is_branch() => head.shorthand().unwrap_or("?").to_owned(),
        Ok(head) => head.target().map_or_else(|| "?".to_owned(), |oid| oid.to_string().chars().take(7).collect()),
        Err(_) => "unborn".to_owned(),
    }
}

fn is_dirty(repo: &Repository) -> bool {
    if repo.is_bare() {
        return false;
    }
    let mut options = StatusOptions::new();
    // Untracked files count as dirty, but walking into untracked directories to
    // count them does not change the answer and costs the most time.
    options.include_untracked(true).recurse_untracked_dirs(false).include_ignored(false);
    repo.statuses(Some(&mut options)).is_ok_and(|statuses| !statuses.is_empty())
}

#[cfg(test)]
#[path = "../tests/core/git.rs"]
mod tests;
