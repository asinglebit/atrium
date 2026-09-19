use std::{
    ffi::OsString,
    path::{Path, PathBuf},
};

/// How far below the root to look. Repos here sit at `<root>/<group>/<repo>`,
/// and stopping early keeps the scan off deep build trees.
const MAX_DEPTH: usize = 3;

/// Directories never worth descending into, whatever they contain.
const SKIP: [&str; 5] = ["node_modules", "target", ".cargo", ".venv", "vendor"];

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

/// Where to look for projects. Overridable so this is not hardcoded to one
/// machine's layout.
pub fn default_root() -> PathBuf {
    root_from(std::env::var_os(ROOT_ENV), std::env::var_os("HOME"))
}

pub const ROOT_ENV: &str = "ATRIUM_PROJECTS";

/// Split out from `default_root` so the choice can be tested without writing to
/// the one environment every test in the process shares.
fn root_from(override_dir: Option<OsString>, home: Option<OsString>) -> PathBuf {
    match (override_dir, home) {
        (Some(dir), _) => PathBuf::from(dir),
        (None, Some(home)) => PathBuf::from(home).join("projects"),
        (None, None) => PathBuf::from("."),
    }
}

/// Every git repository under `root`, named after its own directory. A repo is
/// a leaf: nothing inside one is offered separately.
pub fn discover(root: &Path) -> Vec<Project> {
    let mut found = Vec::new();
    walk(root, 0, &mut found);
    found.sort();
    found
}

fn walk(dir: &Path, depth: usize, found: &mut Vec<Project>) {
    if depth > MAX_DEPTH {
        return;
    }
    if dir.join(".git").exists() {
        if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
            found.push(Project { name: name.to_owned(), path: dir.to_path_buf() });
        }
        return;
    }

    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else { continue };
        // Symlinks are skipped rather than followed, so a link back up the tree
        // cannot send the scan round in circles.
        if !kind.is_dir() {
            continue;
        }
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if name.starts_with('.') || SKIP.contains(&name) {
            continue;
        }
        walk(&entry.path(), depth + 1, found);
    }
}

#[cfg(test)]
#[path = "../tests/core/projects.rs"]
mod tests;
