use super::*;
use crate::core::agent::{AgentSpec, Harness, Status};
use crate::helpers::palette::Theme;
use crate::ipc::wire;

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

/// An agent whose child has already gone: exiting is the one thing no hook can
/// say, so it has to really happen.
fn gone() -> Agent {
    let spec = AgentSpec::new("true", Vec::new(), "/tmp");
    let mut agent = Agent::spawn(&spec, &harness(), &Theme::classic(), 24, 80).expect("pty should open");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while !agent.has_exited() && std::time::Instant::now() < deadline {
        agent.refresh_status(wire::now_ms());
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(agent.has_exited(), "`true` should have exited by now");
    agent
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
    registry.refresh(wire::now_ms());
    assert_eq!(registry.focused().expect("agent").status, Status::Idle);
}

/// Statuses only ever arrive through a report, so a test that wants one sets
/// it the same way the hook feed does.
fn set_status(registry: &mut Registry, index: usize, event: &str) {
    let id = registry.agents()[index].id;
    registry.apply(&Report { agent_id: id, event: event.to_owned(), at: None }, wire::now_ms());
}

#[test]
fn an_atrium_holding_nothing_needs_nothing() {
    assert!(Registry::new().aggregate().is_none());
}

#[test]
fn the_worst_status_is_the_one_that_carries() {
    let mut registry = registry_of(3);
    set_status(&mut registry, 0, "Stop");
    set_status(&mut registry, 1, "UserPromptSubmit");
    set_status(&mut registry, 2, "Notification");
    assert_eq!(registry.aggregate(), Some(Status::NeedsInput), "one agent waiting outranks two that are not");

    set_status(&mut registry, 0, "StopFailure");
    assert_eq!(registry.aggregate(), Some(Status::NeedsInput), "waiting on you outranks even a failure");
}

#[test]
fn still_going_outranks_how_a_turn_ended() {
    // Whether an ending has been seen is @atrium_unseen's to say, so the word
    // only has to tell a window whether anything is waiting or working.
    let mut registry = registry_of(3);
    set_status(&mut registry, 0, "StopFailure");
    set_status(&mut registry, 1, "Stop");
    assert_eq!(registry.aggregate(), Some(Status::Error), "a failure is worse than a finish");

    set_status(&mut registry, 2, "UserPromptSubmit");
    assert_eq!(registry.aggregate(), Some(Status::Working));
}

#[test]
fn what_ended_is_collected_once_across_every_agent() {
    let mut registry = registry_of(2);
    set_status(&mut registry, 0, "Stop");
    set_status(&mut registry, 1, "StopFailure");

    assert_eq!(registry.take_ended(), Some(Status::Error), "a failure among the finishes is what is said");
    assert_eq!(registry.take_ended(), None);
}

#[test]
fn the_agent_on_the_stage_has_been_seen() {
    let mut registry = registry_of(2);
    set_status(&mut registry, 0, "Stop");
    set_status(&mut registry, 1, "Stop");
    registry.focus_at(1);

    registry.see_focused();

    assert!(registry.agents()[0].unseen, "a row you have not put on the stage is still waiting to be looked at");
    assert!(!registry.agents()[1].unseen);
}

#[test]
fn an_atrium_holding_only_dead_agents_needs_nothing() {
    let mut registry = Registry::new();
    registry.push(gone());
    registry.push(gone());
    assert!(registry.aggregate().is_none(), "a finished row is not something to be pulled back to");
}

#[test]
fn a_dead_agent_does_not_drown_out_a_live_one() {
    let mut registry = Registry::new();
    registry.push(gone());
    registry.push(held("/tmp"));
    set_status(&mut registry, 1, "Notification");
    assert_eq!(registry.aggregate(), Some(Status::NeedsInput));
}

#[test]
fn the_counts_say_how_many_are_in_each_state() {
    let mut registry = registry_of(4);
    set_status(&mut registry, 0, "UserPromptSubmit");
    set_status(&mut registry, 1, "UserPromptSubmit");
    set_status(&mut registry, 2, "Notification");
    set_status(&mut registry, 3, "StopFailure");

    let counts = registry.counts();
    assert_eq!(counts.held, 4);
    assert_eq!(counts.working, 2);
    assert_eq!(counts.needs_input, 1);
    assert_eq!(counts.error, 1);
}

#[test]
fn a_dead_agent_is_still_held() {
    let mut registry = Registry::new();
    registry.push(gone());
    assert_eq!(registry.counts().held, 1, "a row is still a row");
}

#[test]
fn one_repository_is_reported_once_however_many_agents_stand_in_it() {
    let mut registry = Registry::new();
    registry.push(held("/tmp"));
    registry.push(held("/tmp"));
    assert_eq!(registry.repos(), vec![PathBuf::from("/tmp")]);
}
