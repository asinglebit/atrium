use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

use crate::core::profile::KNOWN_PROGRAMS;

/// One of the CLIs atrium knows, found on this machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    pub program: &'static str,
    /// Where it was found, which is the only way to tell two installs apart
    /// when a version manager has put a shim in front of one.
    pub path: PathBuf,
}

/// Which of the CLIs atrium knows are installed, in atrium's order rather than
/// whatever order `PATH` happens to be in.
pub fn scan() -> Vec<Found> {
    scan_with(std::env::var_os("PATH"), is_executable)
}

/// The rules, with the lookup handed in. Split out the way `profile::expand` is
/// and for the same reason: `PATH` belongs to the whole test binary, and the
/// suite runs in parallel.
pub fn scan_with(path: Option<OsString>, is_executable: impl Fn(&Path) -> bool) -> Vec<Found> {
    let Some(path) = path else {
        return Vec::new();
    };
    let dirs: Vec<PathBuf> = std::env::split_paths(&path).collect();

    KNOWN_PROGRAMS
        .iter()
        .filter_map(|program| {
            // The first directory wins, which is the answer a shell would give
            // for the same name.
            let path = dirs.iter().map(|dir| dir.join(program)).find(|candidate| is_executable(candidate))?;
            Some(Found { program, path })
        })
        .collect()
}

/// A file that can actually be run. A directory called `claude` is not a
/// claude, and neither is a README with the name.
///
/// `metadata` follows symlinks, because a version manager's shim is one.
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path).is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
}

#[cfg(test)]
#[path = "../tests/core/installed.rs"]
mod tests;
