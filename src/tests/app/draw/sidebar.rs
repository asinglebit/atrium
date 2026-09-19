use super::*;
use crate::core::agent::{Agent, AgentSpec, Harness};
use ratatui::{Terminal, backend::TestBackend};

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
    terminal.draw(|frame| draw(frame, frame.area(), registry, '.')).expect("draw");
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
