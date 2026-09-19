use super::*;
use std::{fs, path::PathBuf};

/// Builds a tree of directories, marking the ones ending in `/.git` as repos.
fn tree(paths: &[&str]) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    for path in paths {
        fs::create_dir_all(root.path().join(path)).expect("mkdir");
    }
    root
}

fn found(root: &Path) -> Vec<String> {
    discover(root).into_iter().map(|project| project.name).collect()
}

#[test]
fn it_finds_repositories_and_ignores_plain_directories() {
    let root = tree(&["personal/atrium/.git", "personal/guitar/.git", "personal/sprites"]);
    assert_eq!(found(root.path()), vec!["atrium", "guitar"]);
}

#[test]
fn a_repository_is_a_leaf() {
    let root = tree(&["personal/atrium/.git", "personal/atrium/vendored/.git"]);
    assert_eq!(found(root.path()), vec!["atrium"], "nothing inside a repo should be offered separately");
}

#[test]
fn results_come_back_sorted() {
    let root = tree(&["w/zebra/.git", "w/alpha/.git", "w/monkey/.git"]);
    assert_eq!(found(root.path()), vec!["alpha", "monkey", "zebra"]);
}

#[test]
fn noisy_directories_are_never_descended_into() {
    let root = tree(&["node_modules/thing/.git", "target/debug/thing/.git", "real/.git"]);
    assert_eq!(found(root.path()), vec!["real"]);
}

#[test]
fn hidden_directories_are_skipped() {
    let root = tree(&[".cache/thing/.git", "real/.git"]);
    assert_eq!(found(root.path()), vec!["real"]);
}

#[test]
fn a_repository_at_the_root_itself_is_found() {
    let root = tree(&[".git"]);
    assert_eq!(discover(root.path()).len(), 1);
}

#[test]
fn a_missing_root_yields_nothing_rather_than_failing() {
    assert!(discover(Path::new("/definitely/not/here")).is_empty());
}

#[test]
fn an_override_wins_over_the_home_directory() {
    assert_eq!(root_from(Some("/srv/code".into()), Some("/home/me".into())), PathBuf::from("/srv/code"));
}

#[test]
fn without_an_override_it_is_projects_under_home() {
    assert_eq!(root_from(None, Some("/home/me".into())), PathBuf::from("/home/me/projects"));
}

#[test]
fn with_neither_it_falls_back_to_the_working_directory() {
    assert_eq!(root_from(None, None), PathBuf::from("."));
}
