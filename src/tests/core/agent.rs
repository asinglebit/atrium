use super::*;
use crate::adapters::{claude::ClaudeCode, codex::Codex, copilot::Copilot};

/// The variable a claude profile's `config_dir` sets, named through the same
/// lookup the code uses rather than written out twice.
fn claude_config_dir() -> &'static str {
    profile::config_dir_env("claude").expect("claude keeps a config dir")
}

#[test]
fn hook_events_map_to_the_status_they_describe() {
    assert_eq!(Status::from_hook_event("SessionStart"), Some(Status::Idle));
    assert_eq!(Status::from_hook_event("UserPromptSubmit"), Some(Status::Working));
    assert_eq!(Status::from_hook_event("Notification"), Some(Status::NeedsInput));
    assert_eq!(Status::from_hook_event("PermissionRequest"), Some(Status::NeedsInput));
    assert_eq!(Status::from_hook_event("Elicitation"), Some(Status::NeedsInput));
    assert_eq!(Status::from_hook_event("Stop"), Some(Status::Done), "a finished turn is something to say, not the absence of anything");
    assert_eq!(Status::from_hook_event("StopFailure"), Some(Status::Error));
}

#[test]
fn an_unknown_event_is_ignored_rather_than_guessed_at() {
    assert_eq!(Status::from_hook_event("PreToolUse"), None);
    // These lift a wait rather than describing one, so they are not here.
    for event in ["PostToolUse", "PostToolUseFailure", "PermissionDenied", "ElicitationResult", INTERRUPT] {
        assert_eq!(Status::from_hook_event(event), None, "{event}");
    }
    assert_eq!(Status::from_hook_event(""), None);
}

/// claude and copilot both fire it on `/clear`, and a row marked exited stops
/// getting keys -- so an agent that cleared its context went deaf.
#[test]
fn session_end_is_not_taken_for_an_exit() {
    assert_eq!(Status::from_hook_event("SessionEnd"), None);
    for status in [Status::Idle, Status::Working, Status::Done] {
        assert_eq!(status.after("SessionEnd"), status);
    }
}

/// The ones that mean "no longer waiting on you", which is the only thing
/// claude ever says about a prompt being answered.
#[test]
fn a_tool_going_ahead_lifts_a_wait() {
    for event in ["PostToolUse", "PostToolUseFailure", "PermissionDenied", "ElicitationResult"] {
        assert_eq!(Status::NeedsInput.after(event), Status::Working, "{event} should have lifted the wait");
    }
}

#[test]
fn lifting_a_wait_that_is_not_there_leaves_the_status_alone() {
    for event in ["PostToolUse", "PostToolUseFailure", "PermissionDenied", "ElicitationResult"] {
        for settled in [Status::Idle, Status::Working, Status::Done, Status::Error, Status::Exited] {
            assert_eq!(settled.after(event), settled, "{event} should have left {} alone", settled.label());
        }
    }
}

#[test]
fn an_interrupt_stops_only_a_turn_still_going() {
    assert_eq!(Status::Working.after(INTERRUPT), Status::Idle);
    assert_eq!(Status::NeedsInput.after(INTERRUPT), Status::Idle);
    for settled in [Status::Idle, Status::Done, Status::Error, Status::Exited] {
        assert_eq!(settled.after(INTERRUPT), settled, "an interrupt should not have touched {}", settled.label());
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
    for status in [Status::Idle, Status::Working, Status::NeedsInput, Status::Done, Status::Error, Status::Exited] {
        assert!(!status.glyph().is_empty());
        assert!(!status.label().is_empty());
    }
}

/// `cat` just sits on its pty, which is all a status test needs from a child.
fn sitting(kind: Box<dyn AgentKind>) -> Agent {
    let mut agent = Agent::spawn(&AgentSpec::new("cat", Vec::new(), "/tmp"), &harness(), &Theme::classic(), 24, 80).expect("pty should open");
    agent.kind = kind;
    agent
}

/// A hook, stamped the way `atrium hook` stamps one.
fn hook(agent: &mut Agent, event: &str, at: u64) {
    let report = Report { agent_id: agent.id, event: event.to_owned(), at: Some(at) };
    agent.apply(&report, at);
}

/// Far enough past the spawn that a stamp here is never mistaken for one from
/// before it.
fn t(offset: u64) -> u64 {
    wire::now_ms() + 60_000 + offset
}

#[test]
fn a_turn_that_ends_is_told_once_and_waits_to_be_seen() {
    let mut agent = sitting(Box::new(adapters::Unknown));
    hook(&mut agent, "UserPromptSubmit", t(0));
    assert_eq!(agent.take_ended(), None);

    hook(&mut agent, "Stop", t(1_000));

    assert_eq!(agent.status, Status::Done);
    assert!(agent.unseen, "nobody has looked at it yet");
    assert_eq!(agent.take_ended(), Some(Status::Done));
    assert_eq!(agent.take_ended(), None, "one ending is said once");
}

#[test]
fn a_failure_outranks_a_finish_in_what_is_said_to_have_ended() {
    let mut agent = sitting(Box::new(adapters::Unknown));
    hook(&mut agent, "StopFailure", t(0));
    hook(&mut agent, "UserPromptSubmit", t(10_000));
    hook(&mut agent, "Stop", t(20_000));

    assert_eq!(agent.take_ended(), Some(Status::Error), "both ended before anyone asked, and the failure is the one to show");
}

#[test]
fn a_hook_that_arrives_late_cannot_undo_a_newer_one() {
    // A fast turn's Stop can connect before its UserPromptSubmit does.
    let mut agent = sitting(Box::new(adapters::Unknown));
    hook(&mut agent, "Stop", t(2_000));
    hook(&mut agent, "UserPromptSubmit", t(1_000));

    assert_eq!(agent.status, Status::Done, "the prompt that started the turn is older than the turn ending");
}

#[test]
fn a_stop_right_after_a_failure_is_the_same_turn_failing() {
    // copilot says errorOccurred, and then agentStop.
    let mut agent = sitting(Box::new(adapters::Unknown));
    hook(&mut agent, "UserPromptSubmit", t(0));
    hook(&mut agent, "StopFailure", t(1_000));
    hook(&mut agent, "Stop", t(1_500));
    assert_eq!(agent.status, Status::Error);

    // A turn claude resumed on its own after a rate limit ends with a Stop too,
    // much later, and that one is a finish.
    hook(&mut agent, "Stop", t(60_000));
    assert_eq!(agent.status, Status::Done);
}

#[test]
fn the_cli_saying_idle_waits_for_the_hook_that_says_how() {
    let mut agent = sitting(Box::new(adapters::Unknown));
    agent.observe(Activity::Busy, t(0), t(0));
    agent.observe(Activity::Idle, t(5_000), t(5_000));
    assert_eq!(agent.status, Status::Working, "idle looks the same finished, failed or stopped");

    // The hook's stamp is taken after the moment it describes, so it can read
    // as a little later than the file's idle -- or a little earlier.
    hook(&mut agent, "Stop", t(5_040));

    assert_eq!(agent.status, Status::Done);
    agent.refresh_status(t(10_000));
    assert_eq!(agent.status, Status::Done, "the settle was answered, so it has nothing left to do");
}

#[test]
fn idle_with_no_hook_behind_it_was_an_interrupt() {
    let mut agent = sitting(Box::new(adapters::Unknown));
    agent.observe(Activity::Busy, t(0), t(0));
    agent.observe(Activity::Idle, t(5_000), t(5_000));

    agent.refresh_status(t(5_000) + SETTLE_MS - 1);
    assert_eq!(agent.status, Status::Working, "still inside the window a Stop could land in");

    agent.refresh_status(t(5_000) + SETTLE_MS);
    assert_eq!(agent.status, Status::Idle);
    assert_eq!(agent.take_ended(), None, "a turn you stopped is not a result waiting to be read");
}

#[test]
fn the_last_turns_stop_loses_to_the_next_turn_starting() {
    // A queued prompt starts the next turn the moment one ends, and the first
    // turn's Stop can land after the file already says busy again.
    let mut agent = sitting(Box::new(adapters::Unknown));
    agent.observe(Activity::Busy, t(0), t(0));
    agent.observe(Activity::Busy, t(5_010), t(5_010));
    hook(&mut agent, "Stop", t(5_060));

    assert_eq!(agent.status, Status::Working);
}

#[test]
fn busy_after_a_turn_ended_is_a_new_turn() {
    let mut agent = sitting(Box::new(adapters::Unknown));
    hook(&mut agent, "Stop", t(0));
    agent.observe(Activity::Busy, t(1_000), t(1_000));
    assert_eq!(agent.status, Status::Working);

    agent.observe(Activity::Waiting, t(2_000), t(2_000));
    assert_eq!(agent.status, Status::NeedsInput);
    agent.observe(Activity::Busy, t(3_000), t(3_000));
    assert_eq!(agent.status, Status::Working, "the answer is the file going busy again");
}

#[test]
fn once_the_cli_speaks_for_itself_only_the_outcomes_are_heard_from_hooks() {
    let mut agent = sitting(Box::new(adapters::Unknown));
    agent.track.self_reported = true;
    agent.observe(Activity::Busy, t(0), t(0));

    hook(&mut agent, "PermissionRequest", t(1_000));
    assert_eq!(agent.status, Status::Working, "the file says what the agent is doing now");

    hook(&mut agent, "Stop", t(2_000));
    assert_eq!(agent.status, Status::Done, "how a turn ended still comes from the hook");
}

#[test]
fn copilots_keys_say_what_its_hooks_do_not() {
    let mut agent = sitting(Box::new(Copilot));
    hook(&mut agent, "UserPromptSubmit", t(0));

    agent.on_key(&[0x1b], t(1_000));
    assert_eq!(agent.status, Status::Idle, "Esc stops a turn");

    hook(&mut agent, "PermissionRequest", t(5_000));
    agent.on_key(b"1", t(6_000));
    assert_eq!(agent.status, Status::Working, "a digit answers a prompt");

    hook(&mut agent, "PermissionRequest", t(7_000));
    agent.on_key(&[0x1b, b'1'], t(8_000));
    assert_eq!(agent.status, Status::NeedsInput, "Alt+1 is not an answer");
    agent.on_key(&[0x1b], t(9_000));
    assert_eq!(agent.status, Status::Idle, "Esc declines a prompt, which ends the turn");
}

#[test]
fn a_tool_starting_well_after_an_esc_means_the_turn_went_on() {
    let mut agent = sitting(Box::new(Copilot));
    hook(&mut agent, "UserPromptSubmit", t(0));
    agent.on_key(&[0x03], t(1_000));
    assert_eq!(agent.status, Status::Idle);

    // One this soon may have been on its way before the key.
    hook(&mut agent, "PreToolUse", t(1_000) + REVIVE_MS);
    assert_eq!(agent.status, Status::Idle);

    hook(&mut agent, "PreToolUse", t(1_000) + REVIVE_MS + HOOK_SLACK_MS + 500);
    assert_eq!(agent.status, Status::Working, "the key closed a menu, not the turn");
}

#[test]
fn a_cli_without_key_rules_is_not_guessed_about() {
    let mut agent = sitting(Box::new(ClaudeCode));
    hook(&mut agent, "UserPromptSubmit", t(0));
    agent.on_key(&[0x1b], t(1_000));
    assert_eq!(agent.status, Status::Working, "claude's own file says when a turn was stopped");
}

/// claude's session file, in the shape claude writes it, with only the fields
/// atrium reads kept -- and a few it does not, to prove they are left alone.
fn session_file(pid: u32, status: &str, waiting_for: Option<&str>, at: u64) -> String {
    let waiting_for = waiting_for.map_or_else(|| "null".to_owned(), |reason| format!("\"{reason}\""));
    format!(r#"{{"pid":{pid},"sessionId":"s","cwd":"/tmp","version":"2.1.283","kind":"interactive","status":"{status}","waitingFor":{waiting_for},"updatedAt":{at},"statusUpdatedAt":{at}}}"#)
}

#[test]
fn claudes_session_file_is_read_whenever_it_changes() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut agent = sitting(Box::new(ClaudeCode));
    let pid = agent.session().pid().expect("pid");
    let path = dir.path().join(format!("{pid}.json"));
    agent.track.file = Some((path.clone(), None));

    std::fs::write(&path, session_file(pid, "busy", None, t(0))).expect("write");
    agent.refresh_status(t(0));
    assert_eq!(agent.status, Status::Working);
    assert!(agent.track.self_reported);

    // Written again inside the same instant, a changed mtime is what says so.
    std::thread::sleep(std::time::Duration::from_millis(20));
    std::fs::write(&path, session_file(pid, "waiting", Some("permission"), t(1_000))).expect("write");
    agent.refresh_status(t(1_000));
    assert_eq!(agent.status, Status::NeedsInput);
}

#[test]
fn a_menu_of_your_own_is_not_claude_waiting_on_you() {
    let mut agent = sitting(Box::new(ClaudeCode));
    let pid = agent.session().pid().expect("pid");
    let reading = agent.kind.read_activity(&session_file(pid, "waiting", Some("dialog open"), t(0)), pid).expect("a file claude wrote");

    assert_eq!(reading.activity, None);
    agent.observe(Activity::Busy, t(0), t(0));
    assert_eq!(agent.status, Status::Working);
}

#[test]
fn a_session_file_left_by_an_earlier_process_is_not_believed() {
    let dir = tempfile::tempdir().expect("tempdir");
    let mut agent = sitting(Box::new(ClaudeCode));
    let pid = agent.session().pid().expect("pid");
    let path = dir.path().join(format!("{pid}.json"));
    agent.track.file = Some((path.clone(), None));

    std::fs::write(&path, session_file(pid, "busy", None, agent.track.born_at - 60_000)).expect("write");
    agent.refresh_status(t(0));

    assert_eq!(agent.status, Status::Idle);
    assert!(!agent.track.self_reported, "the hooks are still what this agent goes by");
}

#[test]
fn a_session_file_for_another_pid_is_not_believed() {
    let agent = sitting(Box::new(ClaudeCode));
    let pid = agent.session().pid().expect("pid");
    assert!(agent.kind.read_activity(&session_file(pid + 1, "busy", None, t(0)), pid).is_none());
    assert!(agent.kind.read_activity("{\"pid\":", pid).is_none(), "half written");
}

#[test]
fn codex_is_read_from_its_title_and_only_when_it_changes() {
    let mut agent = sitting(Box::new(Codex));
    hook(&mut agent, "UserPromptSubmit", t(0));
    let title = |agent: &Agent, text: &str| agent.session().parser().lock().expect("parser").process(format!("\x1b]0;{text}\x07").as_bytes());

    title(&agent, "[ ! ] Action Required");
    agent.refresh_status(t(1_000));
    assert_eq!(agent.status, Status::NeedsInput);

    title(&agent, "[ . ] Action Required");
    agent.refresh_status(t(1_500));
    assert_eq!(agent.status, Status::NeedsInput, "the blink is one wait, not two");

    title(&agent, "\u{280b} Working");
    agent.refresh_status(t(2_000));
    assert_eq!(agent.status, Status::Working, "leaving Action Required is the answer");

    title(&agent, "Ready");
    agent.refresh_status(t(3_000));
    hook(&mut agent, "Stop", t(3_100));
    assert_eq!(agent.status, Status::Done);
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
