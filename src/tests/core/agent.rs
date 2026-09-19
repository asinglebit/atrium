use super::*;

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
    assert_eq!(Status::from_hook_event(""), None);
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
        args: vec!["-c".to_owned(), format!("printf \"dir=${}\"", profile::CONFIG_DIR_ENV)],
        env: vec![(profile::CONFIG_DIR_ENV.to_owned(), "/home/x/.claude-work".to_owned())],
    };

    let agent = Agent::spawn(&AgentSpec::from_profile(&profile, "."), &harness(), 8, 60).expect("pty should open");

    assert!(wait_for(&agent, "dir=/home/x/.claude-work"), "the subscription never reached the agent");
}

#[test]
fn a_spec_from_a_profile_carries_its_args_and_environment() {
    let profile = Profile {
        name: "work".to_owned(),
        program: "claude".to_owned(),
        args: vec!["--append-system-prompt-file".to_owned(), "/etc/prompt.md".to_owned()],
        env: vec![(profile::CONFIG_DIR_ENV.to_owned(), "/home/x/.claude-work".to_owned())],
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
