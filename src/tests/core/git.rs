use super::*;
use git2::{Repository, Signature};
use std::fs;

fn commit_something(repo: &Repository, name: &str) {
    let root = repo.workdir().expect("workdir");
    fs::write(root.join(name), b"x").expect("write");

    let mut index = repo.index().expect("index");
    index.add_path(Path::new(name)).expect("add");
    index.write().expect("write index");
    let tree = repo.find_tree(index.write_tree().expect("write tree")).expect("tree");

    let who = Signature::now("atrium", "atrium@example.com").expect("signature");
    let parents: Vec<git2::Commit> = repo.head().ok().and_then(|h| h.peel_to_commit().ok()).into_iter().collect();
    let parents: Vec<&git2::Commit> = parents.iter().collect();
    repo.commit(Some("HEAD"), &who, &who, "c", &tree, &parents).expect("commit");
}

#[test]
fn a_path_outside_any_repository_has_no_context() {
    let dir = tempfile::tempdir().expect("tempdir");
    assert!(context_for(dir.path()).is_none());
}

#[test]
fn a_fresh_repository_reports_an_unborn_head() {
    let dir = tempfile::tempdir().expect("tempdir");
    Repository::init(dir.path()).expect("init");
    assert_eq!(context_for(dir.path()).expect("context").branch, "unborn");
}

#[test]
fn a_committed_repository_reports_its_branch_and_is_clean() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init");
    commit_something(&repo, "a.txt");

    let context = context_for(dir.path()).expect("context");
    assert!(!context.branch.is_empty());
    assert_ne!(context.branch, "unborn");
    assert!(!context.dirty, "everything is committed, so nothing is dirty");
}

#[test]
fn an_untracked_file_makes_the_tree_dirty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init");
    commit_something(&repo, "a.txt");
    fs::write(dir.path().join("b.txt"), b"new").expect("write");

    assert!(context_for(dir.path()).expect("context").dirty);
}

#[test]
fn a_modified_file_makes_the_tree_dirty() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init");
    commit_something(&repo, "a.txt");
    fs::write(dir.path().join("a.txt"), b"changed").expect("write");

    assert!(context_for(dir.path()).expect("context").dirty);
}

#[test]
fn a_subdirectory_reports_the_repository_it_is_in() {
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(dir.path()).expect("init");
    commit_something(&repo, "a.txt");
    let nested = dir.path().join("src/deep");
    fs::create_dir_all(&nested).expect("mkdir");

    assert!(context_for(&nested).is_some());
}
