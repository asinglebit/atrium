use super::*;
use crate::{
    core::agent::{Agent, AgentSpec, Harness},
    helpers::palette::Theme,
};

fn harness() -> Harness {
    Harness { exe: "atrium".into(), socket: "/nonexistent/atrium.sock".into() }
}

/// Two held agents, both of them working.
fn two_working() -> Registry {
    let mut registry = Registry::new();
    for dir in ["/tmp", "/usr"] {
        let spec = AgentSpec::new("cat", Vec::new(), dir);
        let mut agent = Agent::spawn(&spec, &harness(), &Theme::classic(), 24, 80).expect("pty should open");
        agent.apply_event("UserPromptSubmit");
        registry.push(agent);
    }
    registry
}

/// The colour of a row's status mark, which is the first span on the line.
fn mark_colour(lines: &[Line<'_>], row: usize) -> Option<ratatui::style::Color> {
    lines[row].spans[0].style.fg
}

#[test]
fn a_settled_status_holds_its_colour_through_the_beat() {
    let theme = Theme::classic();
    for status in [Status::Idle, Status::NeedsInput, Status::Done, Status::Error, Status::Exited] {
        for unseen in [true, false] {
            assert_eq!(status_style(&theme, status, unseen, true), status_style(&theme, status, unseen, false), "{status:?} holds still and should ignore the beat");
        }
    }
}

#[test]
fn working_drops_to_grey_on_the_dark_half() {
    let theme = Theme::classic();
    assert_ne!(status_style(&theme, Status::Working, false, true), status_style(&theme, Status::Working, false, false), "working flickers");
    assert_eq!(status_style(&theme, Status::Working, false, false).fg, Some(theme.COLOR_GREY_600));
}

#[test]
fn the_row_on_the_stage_does_not_pulse() {
    let theme = Theme::classic();
    let mut registry = two_working();
    registry.focus_at(0);

    // The dark half of the beat: the row you are not looking at drops to grey,
    // and the one on the stage keeps its colour -- the pulse is there to pull
    // your eye somewhere, and it is already here.
    let dark = agent_lines(&registry, &theme, '.', false, 40);

    assert_eq!(mark_colour(&dark, 0), Some(theme.COLOR_GREY_500), "the focused row should hold the lit grey");
    assert_eq!(mark_colour(&dark, 1), Some(theme.COLOR_GREY_600), "an unfocused row still pulses");
}

#[test]
fn the_pulse_follows_the_focus() {
    let theme = Theme::classic();
    let mut registry = two_working();
    registry.focus_at(1);

    let dark = agent_lines(&registry, &theme, '.', false, 40);

    assert_eq!(mark_colour(&dark, 0), Some(theme.COLOR_GREY_600));
    assert_eq!(mark_colour(&dark, 1), Some(theme.COLOR_GREY_500));
}

#[test]
fn on_the_lit_half_every_row_reads_the_same() {
    let theme = Theme::classic();
    let registry = two_working();

    let lit = agent_lines(&registry, &theme, '.', true, 40);

    assert_eq!(mark_colour(&lit, 0), Some(theme.COLOR_GREY_500));
    assert_eq!(mark_colour(&lit, 1), Some(theme.COLOR_GREY_500));
}

#[test]
fn a_finished_row_is_green_until_it_has_been_on_the_stage() {
    let theme = Theme::classic();
    let mut registry = two_working();
    for index in 0..2 {
        let id = registry.agents()[index].id;
        registry.apply(&crate::ipc::wire::Report { agent_id: id, event: "Stop".to_owned(), at: None }, crate::ipc::wire::now_ms());
    }
    registry.focus_at(0);
    registry.see_focused();

    let lines = agent_lines(&registry, &theme, '.', true, 40);

    assert_eq!(mark_colour(&lines, 0), Some(theme.COLOR_GREY_600), "the row on the stage has been seen");
    assert_eq!(mark_colour(&lines, 1), Some(theme.COLOR_GREEN), "the other one's answer is still waiting to be read");
}
