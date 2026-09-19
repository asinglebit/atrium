use super::*;
use crate::{
    core::agent::{Agent, AgentSpec, Harness},
    helpers::palette::Theme,
};
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

fn harness() -> Harness {
    Harness { exe: "atrium".into(), socket: "/nonexistent/atrium.sock".into() }
}

fn registry_of(dirs: &[&str]) -> Registry {
    let mut registry = Registry::new();
    for dir in dirs {
        let spec = AgentSpec::new("cat", Vec::new(), *dir);
        registry.push(Agent::spawn(&spec, &harness(), 24, 80).expect("pty should open"));
    }
    registry
}

fn rendered(registry: &Registry) -> String {
    let mut terminal = Terminal::new(TestBackend::new(26, 6)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), registry, &Theme::classic(), '.', 0)).expect("draw");
    terminal.backend().buffer().content().chunks(26).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

#[test]
fn rows_are_named_after_their_directory() {
    let registry = registry_of(&["/tmp", "/usr"]);
    let out = rendered(&registry);

    assert!(out.contains("tmp"), "expected a tmp row in:\n{out}");
    assert!(out.contains("usr"), "expected a usr row in:\n{out}");
}

#[test]
fn the_first_nine_rows_carry_a_jump_key() {
    let registry = registry_of(&["/tmp", "/usr"]);
    let out = rendered(&registry);

    assert!(out.contains("1 tmp"), "expected row 1 to be numbered in:\n{out}");
    assert!(out.contains("2 usr"), "expected row 2 to be numbered in:\n{out}");
}

#[test]
fn the_header_counts_what_is_held() {
    let registry = registry_of(&["/tmp", "/usr", "/etc"]);
    assert!(rendered(&registry).contains("agents 3"));
}

#[test]
fn an_empty_registry_still_draws_its_frame() {
    let registry = Registry::new();
    assert!(rendered(&registry).contains("agents 0"));
}

/// A real repository on a named branch, so a row has a branch to line up.
fn repo_on(branch: &str) -> tempfile::TempDir {
    use git2::{Repository, Signature};
    let dir = tempfile::tempdir().expect("tempdir");
    let repo = Repository::init(&dir).expect("init");

    std::fs::write(dir.path().join("a.txt"), b"x").expect("write");
    let mut index = repo.index().expect("index");
    index.add_path(std::path::Path::new("a.txt")).expect("add");
    index.write().expect("write index");
    let tree = repo.find_tree(index.write_tree().expect("tree oid")).expect("tree");
    let who = Signature::now("atrium", "a@b.c").expect("signature");
    repo.commit(Some("HEAD"), &who, &who, "c", &tree, &[]).expect("commit");

    let head = repo.head().expect("head").peel_to_commit().expect("commit");
    repo.branch(branch, &head, true).expect("branch");
    repo.set_head(&format!("refs/heads/{branch}")).expect("set head");
    dir
}

#[test]
fn branches_share_one_right_aligned_column() {
    let short = repo_on("main");
    let long = repo_on("a-much-longer-branch");

    let mut registry = Registry::new();
    for dir in [short.path(), long.path()] {
        let spec = AgentSpec::new("cat", Vec::new(), dir);
        registry.push(Agent::spawn(&spec, &harness(), 24, 80).expect("pty"));
    }

    let mut terminal = Terminal::new(TestBackend::new(40, 5)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), &registry, &Theme::classic(), '.', 0)).expect("draw");
    let rows: Vec<String> = terminal.backend().buffer().content().chunks(40).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect();

    // Rows 1 and 2 are the two agents; their branches must end at the same column.
    let end_of = |row: &String| row.trim_end().chars().count();
    assert_eq!(end_of(&rows[1]), end_of(&rows[2]), "branches are ragged:\n{}\n{}", rows[1], rows[2]);
    assert!(rows[1].contains("main"), "{}", rows[1]);
}

#[test]
fn a_click_lands_on_the_row_under_it() {
    let area = Rect::new(0, 4, 26, 10);
    // Row 4 is the title, so the first agent is on row 5.
    assert_eq!(row_at(area, 0, 4), None, "the title line is not an agent");
    assert_eq!(row_at(area, 0, 5), Some(0));
    assert_eq!(row_at(area, 0, 7), Some(2));
}

#[test]
fn a_click_accounts_for_how_far_the_list_is_scrolled() {
    let area = Rect::new(0, 4, 26, 10);
    assert_eq!(row_at(area, 3, 5), Some(3), "the top row is row 3 once scrolled by three");
}

#[test]
fn a_click_below_the_pane_lands_nowhere() {
    let area = Rect::new(0, 4, 26, 10);
    assert_eq!(row_at(area, 0, 14), None);
    assert_eq!(row_at(area, 0, 99), None);
}
