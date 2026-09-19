use super::*;
use crate::core::agent::{AgentSpec, Harness, Status};
use crate::helpers::palette::Theme;

/// `cat` just sits on its pty, which is all a registry test needs from a child.
/// Tests never bind a socket; a path that cannot be connected to is exactly
/// what a hook is expected to shrug off.
fn harness() -> Harness {
    Harness { exe: "atrium".into(), socket: "/nonexistent/atrium.sock".into() }
}

fn held(cwd: &str) -> Agent {
    let spec = AgentSpec::new("cat", Vec::new(), cwd);
    Agent::spawn(&spec, &harness(), &Theme::classic(), 24, 80).expect("pty should open")
}

fn registry_of(n: usize) -> Registry {
    let mut registry = Registry::new();
    for _ in 0..n {
        registry.push(held("/tmp"));
    }
    registry
}

#[test]
fn a_new_registry_holds_nothing() {
    let registry = Registry::new();
    assert!(registry.is_empty());
    assert!(registry.focused().is_none());
}

#[test]
fn a_newly_held_agent_takes_the_stage() {
    let mut registry = registry_of(2);
    assert_eq!(registry.focus(), 1);

    registry.push(held("/tmp"));
    assert_eq!(registry.focus(), 2);
    assert_eq!(registry.len(), 3);
}

#[test]
fn focus_wraps_in_both_directions() {
    let mut registry = registry_of(3);
    registry.focus_at(0);

    registry.focus_prev();
    assert_eq!(registry.focus(), 2, "back from the first should wrap to the last");

    registry.focus_next();
    assert_eq!(registry.focus(), 0, "forward from the last should wrap to the first");
}

#[test]
fn jumping_past_the_end_is_ignored() {
    let mut registry = registry_of(2);
    registry.focus_at(0);

    registry.focus_at(7);
    assert_eq!(registry.focus(), 0, "an out-of-range jump should not move the stage");
}

#[test]
fn dismissing_keeps_the_focus_in_range() {
    let mut registry = registry_of(3);
    assert_eq!(registry.focus(), 2);

    registry.dismiss_focused();
    assert_eq!(registry.len(), 2);
    assert_eq!(registry.focus(), 1, "focus should fall back onto the new last row");

    registry.dismiss_focused();
    registry.dismiss_focused();
    assert!(registry.is_empty());
    assert!(registry.focused().is_none());
}

#[test]
fn dismissing_from_the_middle_keeps_showing_something() {
    let mut registry = registry_of(3);
    registry.focus_at(1);

    registry.dismiss_focused();
    assert_eq!(registry.len(), 2);
    assert!(registry.focused().is_some());
}

#[test]
fn dismissing_an_empty_registry_is_harmless() {
    let mut registry = Registry::new();
    assert!(registry.dismiss_focused().is_none());
}

#[test]
fn resize_reaches_every_agent_not_just_the_focused_one() {
    let mut registry = registry_of(3);
    registry.resize_all(40, 120).expect("resize should succeed");

    for agent in registry.agents() {
        let parser = agent.session().parser().lock().expect("parser lock");
        assert_eq!(parser.screen().size(), (40, 120));
    }
}

#[test]
fn a_live_agent_is_not_reported_as_exited() {
    let mut registry = registry_of(1);
    registry.refresh();
    assert_eq!(registry.focused().expect("agent").status, Status::Idle);
}
