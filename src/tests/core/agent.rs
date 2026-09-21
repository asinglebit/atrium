use super::*;

/// The variable a claude profile's `config_dir` sets, named through the same
/// lookup the code uses rather than written out twice.
fn claude_config_dir() -> &'static str {
    profile::config_dir_env("claude").expect("claude keeps a config dir")
}

#[test]
fn hook_events_map_to_the_status_they_describe() {
    assert_eq!(Status::from_hook_event("SessionStart"), Some(Status::Idle));
    assert_eq!(Status::from_hook_event("Stop"), Some(Status::Idle));
    assert_eq!(Status::from_hook_event("UserPromptSubmit"), Some(Status::Working));
    assert_eq!(Status::from_hook_event("Notification"), Some(Status::NeedsInput));
    assert_eq!(Status::from_hook_event("PermissionRequest"), Some(Status::NeedsInput));
    assert_eq!(Status::from_hook_event("StopFailure"), Some(Status::Error));
    assert_eq!(Status::from_hook_event("SessionEnd"), Some(Status::Exited));
}

#[test]
fn an_unknown_event_is_ignored_rather_than_guessed_at() {
    assert_eq!(Status::from_hook_event("PreToolUse"), None);
    // These two lift a wait rather than describing one, so they are not here.
    assert_eq!(Status::from_hook_event("PostToolUse"), None);
    assert_eq!(Status::from_hook_event("PermissionDenied"), None);
    assert_eq!(Status::from_hook_event(""), None);
}

/// The two that mean "no longer waiting on you", which is the only thing
/// Claude ever says about a permission prompt being answered.
#[test]
fn a_tool_going_ahead_lifts_a_wait() {
    for event in ["PostToolUse", "PermissionDenied"] {
        assert_eq!(Status::NeedsInput.after(event), Status::Working, "{event} should have lifted the wait");
    }
}

#[test]
fn lifting_a_wait_that_is_not_there_leaves_the_status_alone() {
    for event in ["PostToolUse", "PermissionDenied"] {
        for settled in [Status::Idle, Status::Working, Status::Error, Status::Exited] {
            assert_eq!(settled.after(event), settled, "{event} should have left {} alone", settled.label());
        }
    }
}

#[test]
fn an_agent_that_has_exited_stays_exited() {
    for event in ["SessionStart", "UserPromptSubmit", "Notification", "Stop"] {
        assert_eq!(Status::Exited.after(event), Status::Exited, "{event} should not have revived a dead row");
    }
}

#[test]
fn an_event_that_says_nothing_leaves_the_status_where_it_was() {
    assert_eq!(Status::NeedsInput.after("PreToolUse"), Status::NeedsInput);
    assert_eq!(Status::Working.after(""), Status::Working);
}

#[test]
fn every_status_has_a_glyph_and_a_label() {
    for status in [Status::Idle, Status::Working, Status::NeedsInput, Status::Error, Status::Exited] {
        assert!(!status.glyph().is_empty());
        assert!(!status.label().is_empty());
    }
}

#[test]
fn a_spec_is_named_after_its_directory() {
    assert_eq!(AgentSpec::new("claude", Vec::new(), "/home/me/projects/bazzite").name(), "bazzite");
}

#[test]
fn a_spec_with_no_directory_name_falls_back_to_the_program() {
    assert_eq!(AgentSpec::new("claude", Vec::new(), "/").name(), "claude");
}

use crate::core::profile;
use crate::helpers::palette::Theme;
use std::time::{Duration, Instant};

fn harness() -> Harness {
    Harness { exe: "atrium".into(), socket: "/nonexistent/atrium.sock".into() }
}

/// Spins until the agent's screen says what we are waiting for, or gives up.
fn wait_for(agent: &Agent, needle: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(parser) = agent.session().parser().lock()
            && parser.screen().contents().contains(needle)
        {
            return true;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn a_profiles_environment_reaches_the_child() {
    // The whole point of a profile: this variable is which subscription is used.
    let profile = Profile {
        name: "work".to_owned(),
        program: "sh".to_owned(),
        args: vec!["-c".to_owned(), format!("printf \"dir=${}\"", claude_config_dir())],
        env: vec![(claude_config_dir().to_owned(), "/home/x/.claude-work".to_owned())],
    };

    let agent = Agent::spawn(&AgentSpec::from_profile(&profile, "."), &harness(), &Theme::classic(), 8, 60).expect("pty should open");

    assert!(wait_for(&agent, "dir=/home/x/.claude-work"), "the subscription never reached the agent");
}

#[test]
fn a_spec_from_a_profile_carries_its_args_and_environment() {
    let profile = Profile {
        name: "work".to_owned(),
        program: "claude".to_owned(),
        args: vec!["--append-system-prompt-file".to_owned(), "/etc/prompt.md".to_owned()],
        env: vec![(claude_config_dir().to_owned(), "/home/x/.claude-work".to_owned())],
    };

    let spec = AgentSpec::from_profile(&profile, "/home/x/projects/atrium");

    assert_eq!(spec.program, "claude");
    assert_eq!(spec.args, profile.args);
    assert_eq!(spec.env, profile.env);
    assert_eq!(spec.profile.as_deref(), Some("work"));
    assert_eq!(spec.name(), "atrium", "the row is still named after the directory");
}

#[test]
fn a_bare_cli_leaves_the_row_untagged() {
    let spec = AgentSpec::from_profile(&Profile::bare("claude"), "/home/x/projects/atrium");

    assert_eq!(spec.profile, None, "every row would carry the same tag, which says nothing");
    assert!(spec.env.is_empty());
}

#[test]
fn a_command_given_on_the_command_line_picks_up_no_profile() {
    let spec = AgentSpec::new("bash", vec!["--norc".to_owned()], "/tmp");

    assert!(spec.env.is_empty(), "`atrium bash` should not inherit a subscription");
    assert_eq!(spec.profile, None);
}
