use std::{
    ffi::OsString,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    thread::{self, JoinHandle},
};

/// The command to run when a worktree appears, when anything is listening.
/// Neutral rather than `ATRIUM_*` because guitar reads the same variable.
pub const HOOK_ENV: &str = "WORKTREE_HOOK";

/// What happened, as the hook's first argument. The path says the rest.
pub const CREATED: &str = "created";

/// Say that a worktree appeared. The mirror of `atrium hook`: nothing listening
/// is the ordinary case, and a missed announcement is not worth disturbing
/// anything over.
pub fn worktree_created(path: &Path) {
    let _ = worktree_created_with(std::env::var_os(HOOK_ENV), path);
}

/// Split out so a test can choose the hook without writing to the one
/// environment every test in the process shares. The handle is what lets a
/// test wait for the hook it asked for; the caller above drops it.
pub fn worktree_created_with(hook: Option<OsString>, path: &Path) -> Option<JoinHandle<()>> {
    // Nothing to announce to, and that is fine.
    let hook = hook.filter(|hook| !hook.is_empty())?;
    Some(announce(hook, path.to_path_buf()))
}

/// The same announcement, waited for. A process that is about to exit would
/// otherwise take the thread down with it before the hook had a chance to run.
pub fn worktree_created_now(path: &Path) {
    if let Some(handle) = worktree_created_with(std::env::var_os(HOOK_ENV), path) {
        let _ = handle.join();
    }
}

/// Run on a thread of its own, so a hook that sleeps cannot hold up a frame --
/// and waited on there, so it leaves no zombie behind the way a bare spawn
/// would. The same shape as the one short-lived thread per hook in `ipc`.
fn announce(hook: OsString, path: PathBuf) -> JoinHandle<()> {
    thread::spawn(move || {
        let _ = Command::new(hook).arg(CREATED).arg(path).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
    })
}

#[cfg(test)]
#[path = "../tests/core/notify.rs"]
mod tests;
