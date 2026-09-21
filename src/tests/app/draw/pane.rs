use super::*;
use crate::{
    core::agent::{Agent, AgentSpec, Harness},
    helpers::palette::Theme,
};

fn harness() -> Harness {
    Harness { exe: "atrium".into(), socket: "/nonexistent/atrium.sock".into() }
}

/// Two held agents, both of them waiting on you.
fn two_waiting() -> Registry {
    let mut registry = Registry::new();
    for dir in ["/tmp", "/usr"] {
        let spec = AgentSpec::new("cat", Vec::new(), dir);
        let mut agent = Agent::spawn(&spec, &harness(), &Theme::classic(), 24, 80).expect("pty should open");
        agent.apply_event("Notification");
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
    for status in [Status::Idle, Status::Error, Status::Exited] {
        assert_eq!(status_style(&theme, status, true), status_style(&theme, status, false), "{status:?} is settled and should ignore the beat");
    }
}

#[test]
fn a_waiting_status_drops_to_grey_on_the_dark_half() {
    let theme = Theme::classic();
    for status in [Status::Working, Status::NeedsInput] {
        assert_ne!(status_style(&theme, status, true), status_style(&theme, status, false), "{status:?} is still waiting and should pulse");
        assert_eq!(status_style(&theme, status, false).fg, Some(theme.COLOR_GREY_600));
    }
}

#[test]
fn the_row_on_the_stage_does_not_pulse() {
    let theme = Theme::classic();
    let mut registry = two_waiting();
    registry.focus_at(0);

    // The dark half of the beat: the row you are not looking at drops to grey,
    // and the one on the stage keeps its colour -- the pulse is there to pull
    // your eye somewhere, and it is already here.
    let dark = agent_lines(&registry, &theme, '.', false, 40);

    assert_eq!(mark_colour(&dark, 0), Some(theme.COLOR_BLUE), "the focused row should hold its colour");
    assert_eq!(mark_colour(&dark, 1), Some(theme.COLOR_GREY_600), "an unfocused row still pulses");
}

#[test]
fn the_pulse_follows_the_focus() {
    let theme = Theme::classic();
    let mut registry = two_waiting();
    registry.focus_at(1);

    let dark = agent_lines(&registry, &theme, '.', false, 40);

    assert_eq!(mark_colour(&dark, 0), Some(theme.COLOR_GREY_600));
    assert_eq!(mark_colour(&dark, 1), Some(theme.COLOR_BLUE));
}

#[test]
fn on_the_lit_half_every_row_reads_the_same() {
    let theme = Theme::classic();
    let registry = two_waiting();

    let lit = agent_lines(&registry, &theme, '.', true, 40);

    assert_eq!(mark_colour(&lit, 0), Some(theme.COLOR_BLUE));
    assert_eq!(mark_colour(&lit, 1), Some(theme.COLOR_BLUE));
}
