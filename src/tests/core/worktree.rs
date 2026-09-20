use super::*;
use git2::Signature;
use std::fs;

/// A repository with one commit, which is the least a worktree can be cut from.
fn repo_with_a_commit(root: &Path) -> Repository {
    let repo = Repository::init(root).expect("init");
    fs::write(root.join("a.txt"), b"x").expect("write");

    // Scoped so the index and tree are dropped before the repository they
    // borrow is handed back.
    {
        let mut index = repo.index().expect("index");
        index.add_path(Path::new("a.txt")).expect("add");
        index.write().expect("write index");
        let tree = repo.find_tree(index.write_tree().expect("write tree")).expect("tree");

        let who = Signature::now("atrium", "atrium@example.com").expect("signature");
        repo.commit(Some("HEAD"), &who, &who, "c", &tree, &[]).expect("commit");
    }
    repo
}

#[test]
fn a_name_that_would_change_what_a_path_means_is_refused() {
    for name in ["", ".", "..", "a/b", "a\\b"] {
        assert!(!is_valid_name(name), "{name:?} should be refused");
    }
}

#[test]
fn an_ordinary_name_is_allowed() {
    for name in ["scratch", "fix-42", "DT-21012"] {
        assert!(is_valid_name(name));
    }
}

#[test]
fn a_worktree_goes_beside_the_repository_named_for_both() {
    assert_eq!(default_path(Path::new("/projects/personal/atrium"), "scratch"), PathBuf::from("/projects/personal/atrium-scratch"));
}

#[test]
fn creating_one_makes_a_branch_of_the_same_name() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);

    let path = default_path(&root, "scratch");
    let made = create(&repo, "scratch", &path).expect("create");

    assert_eq!(made, path);
    assert!(path.join("a.txt").exists(), "the worktree is checked out");
    assert!(repo.find_branch("scratch", BranchType::Local).is_ok(), "the branch is made too");
    assert_eq!(list(&repo).expect("list"), vec![path]);
}

#[test]
fn a_bad_name_is_refused_before_anything_is_written() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);

    assert!(create(&repo, "a/b", &root.join("x")).is_err());
    assert!(list(&repo).expect("list").is_empty());
}

#[test]
fn a_worktree_that_cannot_be_made_leaves_no_branch_behind() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);

    // A path whose parent is not there is the cheapest way to make the worktree
    // fail after the branch has already been made.
    assert!(create(&repo, "doomed", Path::new("/nonexistent/deep/doomed")).is_err());
    assert!(repo.find_branch("doomed", BranchType::Local).is_err(), "the orphan branch is cleaned up");
}

#[test]
fn a_name_already_taken_does_not_delete_the_branch_that_took_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);
    create(&repo, "scratch", &default_path(&root, "scratch")).expect("create");

    assert!(create(&repo, "scratch", &root.parent().expect("parent").join("other")).is_err());
    assert!(repo.find_branch("scratch", BranchType::Local).is_ok(), "the branch was not ours to roll back");
}

#[test]
fn the_first_look_at_a_repository_announces_nothing() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);
    create(&repo, "already-here", &default_path(&root, "already-here")).expect("create");

    let mut watch = Watch::new();
    assert!(watch.tick(std::slice::from_ref(&root)).is_empty(), "what was already there is not news");
}

#[test]
fn a_worktree_that_appears_is_noticed_once() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);

    let mut watch = Watch::new();
    watch.tick(std::slice::from_ref(&root));

    let path = default_path(&root, "later");
    create(&repo, "later", &path).expect("create");

    assert_eq!(watch.tick(std::slice::from_ref(&root)), vec![path], "it is reported the first time");
    assert!(watch.tick(std::slice::from_ref(&root)).is_empty(), "and not again");
}

#[test]
fn an_agent_standing_in_a_worktree_watches_the_same_set() {
    let dir = tempfile::tempdir().expect("tempdir");
    let root = dir.path().join("repo");
    fs::create_dir(&root).expect("mkdir");
    let repo = repo_with_a_commit(&root);
    let inside = default_path(&root, "inside");
    create(&repo, "inside", &inside).expect("create");

    let mut watch = Watch::new();
    // Seeded from the worktree, then looked at from the main checkout: one
    // repository, so the second look must not rediscover the first's set.
    watch.tick(&[inside]);
    assert!(watch.tick(std::slice::from_ref(&root)).is_empty());
}

#[test]
fn a_path_that_is_not_a_repository_is_skipped() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut watch = Watch::new();
    assert!(watch.tick(&[dir.path().to_path_buf()]).is_empty());
}
