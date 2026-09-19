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
fn the_top_line_is_an_agent_rather_than_a_heading() {
    let registry = registry_of(&["/tmp", "/usr"]);
    let out = rendered(&registry);

    assert!(out.lines().next().is_some_and(|first| first.contains("tmp")), "the first agent should be on the first line:\n{out}");
}

#[test]
fn an_empty_registry_still_draws_the_line_between_the_panes() {
    let registry = Registry::new();
    let out = rendered(&registry);

    assert!(out.contains('\u{2502}'), "the separator should survive an empty list:\n{out}");
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

    // Rows 0 and 1 are the two agents; their branches must end at the same column.
    let end_of = |row: &String| row.trim_end().chars().count();
    assert_eq!(end_of(&rows[0]), end_of(&rows[1]), "branches are ragged:\n{}\n{}", rows[0], rows[1]);
    assert!(rows[0].contains("main"), "{}", rows[0]);
}

#[test]
fn a_click_lands_on_the_row_under_it() {
    let area = Rect::new(0, 4, 26, 10);
    // Nothing is spent on a heading, so the first agent is on the pane's own top row.
    assert_eq!(row_at(area, 0, 3), None, "a click above the pane is not an agent");
    assert_eq!(row_at(area, 0, 4), Some(0));
    assert_eq!(row_at(area, 0, 6), Some(2));
}

#[test]
fn a_click_accounts_for_how_far_the_list_is_scrolled() {
    let area = Rect::new(0, 4, 26, 10);
    assert_eq!(row_at(area, 3, 4), Some(3), "the top row is row 3 once scrolled by three");
}

#[test]
fn a_click_below_the_pane_lands_nowhere() {
    let area = Rect::new(0, 4, 26, 10);
    assert_eq!(row_at(area, 0, 14), None);
    assert_eq!(row_at(area, 0, 99), None);
}

/// The same registry, but held under a named profile the way a subscription is.
fn registry_under(entries: &[(&str, &str)]) -> Registry {
    let mut registry = Registry::new();
    for (dir, name) in entries {
        let profile = crate::core::profile::Profile { name: (*name).to_owned(), program: "cat".to_owned(), args: Vec::new(), env: Vec::new() };
        let spec = AgentSpec::from_profile(&profile, *dir);
        registry.push(Agent::spawn(&spec, &harness(), 24, 80).expect("pty should open"));
    }
    registry
}

#[test]
fn a_row_says_which_profile_it_is_held_under() {
    let registry = registry_under(&[("/tmp", "work"), ("/usr", "personal")]);
    let out = rendered(&registry);

    assert!(out.contains("work"), "expected the work tag in:\n{out}");
    assert!(out.contains("personal"), "expected the personal tag in:\n{out}");
}

#[test]
fn two_agents_on_one_project_are_told_apart_by_their_profile() {
    let registry = registry_under(&[("/tmp", "work"), ("/tmp", "personal")]);
    let rows: Vec<String> = rendered(&registry).lines().take(2).map(str::to_owned).collect();

    assert_ne!(rows[0], rows[1], "identical rows are the problem profiles exist to solve:\n{rows:?}");
}

/// The row without its last column, which is the gutter the scrollbar rides on.
fn row_body(out: &str) -> String {
    let mut chars: Vec<char> = out.lines().next().unwrap_or_default().chars().collect();
    chars.pop();
    chars.into_iter().collect::<String>().trim_end().to_owned()
}

#[test]
fn no_profile_anywhere_costs_the_row_no_columns() {
    let plain = rendered(&registry_of(&["/tmp"]));
    let tagged = rendered(&registry_under(&[("/tmp", "work")]));

    assert!(!plain.contains("work"));
    assert!(row_body(&plain).ends_with("tmp"), "nothing should follow the name when no agent has a profile:\n{plain}");
    assert!(row_body(&tagged).ends_with("work"), "the tag belongs after the name:\n{tagged}");
}
