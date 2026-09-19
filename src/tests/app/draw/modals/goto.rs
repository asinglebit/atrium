use super::*;
use crate::core::agent::{Agent, AgentSpec, Harness};
use crate::helpers::palette::Theme;
use ratatui::{Terminal, backend::TestBackend};

fn harness() -> Harness {
    Harness { exe: "atrium".into(), socket: "/nonexistent/atrium.sock".into() }
}

fn registry_of(dirs: &[&str]) -> Registry {
    let mut registry = Registry::new();
    for dir in dirs {
        let spec = AgentSpec::new("cat", Vec::new(), *dir);
        registry.push(Agent::spawn(&spec, &harness(), &Theme::classic(), 24, 80).expect("pty"));
    }
    registry
}

fn rendered(registry: &Registry, goto: &Goto) -> String {
    let mut terminal = Terminal::new(TestBackend::new(60, 14)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), goto, registry, &Theme::classic(), '.')).expect("draw");
    terminal.backend().buffer().content().chunks(60).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

#[test]
fn it_lists_every_held_agent_with_its_number() {
    let registry = registry_of(&["/tmp", "/usr", "/etc"]);
    let out = rendered(&registry, &Goto::new(registry.len(), 0));

    assert!(out.contains("go to"), "{out}");
    for (index, name) in ["tmp", "usr", "etc"].iter().enumerate() {
        assert!(out.contains(&format!("{} {name}", index + 1)), "row {} missing in:\n{out}", index + 1);
    }
}

#[test]
fn it_says_how_to_use_itself() {
    let registry = registry_of(&["/tmp"]);
    assert!(rendered(&registry, &Goto::new(1, 0)).contains("1-9 jump"));
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    let registry = registry_of(&["/tmp", "/usr"]);
    let goto = Goto::new(2, 0);
    let mut terminal = Terminal::new(TestBackend::new(16, 4)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), &goto, &registry, &Theme::classic(), '.')).expect("should not panic on a small frame");
}
