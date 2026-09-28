use super::*;
use crate::core::agent::Status;

#[test]
fn every_registered_hook_means_something() {
    for hook in HOOKS {
        // Either it names a status, or it lifts a wait -- or, for a tool
        // starting, it is what tells a turn Esc stopped from one that went on.
        let names_a_status = Status::from_hook_event(hook.word).is_some();
        let lifts_a_wait = Status::NeedsInput.after(hook.word) != Status::NeedsInput;
        assert!(names_a_status || lifts_a_wait || hook.word == "PreToolUse", "{} translates to {}, which means nothing", hook.event, hook.word);
    }
}

#[test]
fn every_status_a_turn_can_reach_has_an_event_that_reaches_it() {
    // The whole point of wiring copilot's hooks up: a row that can only ever
    // read idle would be no better than watching the process. Exited is not
    // here, because only the child going away says so.
    for status in [Status::Idle, Status::Working, Status::NeedsInput, Status::Done, Status::Error] {
        assert!(HOOKS.iter().any(|hook| Status::from_hook_event(hook.word) == Some(status)), "nothing copilot reports means {status:?}");
    }
}

#[test]
fn session_end_is_not_registered() {
    // copilot fires it on /clear as well, and a row taken for exited stops
    // getting keys.
    assert!(!HOOKS.iter().any(|hook| hook.event == "sessionEnd"));
    assert!(!hooks_json("/usr/bin/atrium").contains("sessionEnd"));
}

#[test]
fn the_hooks_name_every_registered_event_once() {
    let json = hooks_json("/usr/bin/atrium");
    for hook in HOOKS {
        assert_eq!(json.matches(&format!(r#""{}":"#, hook.event)).count(), 1, "{} should be one key holding all of its entries", hook.event);
        assert!(json.contains(&format!("hook {}", hook.word)), "{} not passed to the handler", hook.event);
    }
}

#[test]
fn only_the_notifications_that_mean_you_are_registered() {
    // Registered bare, a background shell finishing turned the row orange.
    let notifications: Vec<&Hook> = HOOKS.iter().filter(|hook| hook.event == "notification").collect();
    assert!(!notifications.is_empty());
    for hook in notifications {
        let matcher = hook.matcher.expect("never bare");
        assert!(matcher.starts_with('^') && matcher.ends_with('$'), "anchored, so a type merely containing one of these does not count: {matcher}");
        for noise in ["shell_completed", "agent_idle", "agent_completed"] {
            assert!(!matcher.contains(noise), "{noise}");
        }
    }
}

#[test]
fn a_question_and_any_tool_are_told_apart() {
    let tools: Vec<&Hook> = HOOKS.iter().filter(|hook| hook.event == "preToolUse").collect();
    assert_eq!(tools.len(), 2);
    assert!(tools.iter().any(|hook| hook.matcher == Some("^ask_user$") && hook.word == "PermissionRequest"), "ask_user is copilot asking you something");
    assert!(tools.iter().any(|hook| hook.matcher.is_none() && hook.word == "PreToolUse"));
}

#[test]
fn only_an_error_copilot_does_not_get_past_is_a_failure() {
    let json = hooks_json("/usr/bin/atrium");
    let entry = json.lines().find(|line| line.contains("errorOccurred")).expect("errorOccurred is registered");

    assert!(entry.contains(r#"recoverable\"[[:space:]]*:[[:space:]]*false"#), "{entry}");
    assert!(!entry.contains("grep -q") && !entry.contains("grep -Eq"), "without -q grep reads all of stdin, so copilot never writes into a closed pipe: {entry}");
    assert!(entry.contains("exit 0"), "a hook that fails is a hook that interrupts the agent: {entry}");
}

#[test]
fn a_space_in_the_path_stays_one_word() {
    let json = hooks_json("/home/a b/atrium");
    assert!(json.contains(r#"'/home/a b/atrium' hook Stop"#), "got: {json}");
}

#[test]
fn a_quote_in_the_path_cannot_break_out_of_the_shell_string() {
    // copilot's command is a shell string, so this is the hazard claude's exec
    // form does not have. Quoted, the shell reads one word and runs no part of
    // it.
    let json = hooks_json("/tmp/ev'il; rm -rf /");
    assert!(json.contains(r#"'/tmp/ev'\\''il; rm -rf /' hook Stop"#), "got: {json}");
}

#[test]
fn a_quote_in_the_path_cannot_escape_its_json_string_either() {
    let json = hooks_json(r#"/tmp/ev"il"#);
    assert!(json.contains(r#"'/tmp/ev\"il' hook Stop"#), "got: {json}");
}

#[test]
fn only_bash_is_written_because_the_socket_is_unix_only() {
    let json = hooks_json("/usr/bin/atrium");
    assert!(!json.contains("powershell"), "{json}");
    assert_eq!(json.matches(r#""type": "command""#).count(), HOOKS.len());
}

#[test]
fn the_keys_say_what_copilot_does_not() {
    let meaning = |status, bytes: &[u8]| Copilot.key_meaning(status, bytes);

    assert_eq!(meaning(Status::Working, &[0x1b]), Some(KeyMeaning::Stop), "Esc stops a turn");
    assert_eq!(meaning(Status::Working, &[0x03]), Some(KeyMeaning::Stop), "and so does Ctrl-C");
    assert_eq!(meaning(Status::NeedsInput, &[0x1b]), Some(KeyMeaning::Stop), "Esc declines a prompt");
    assert_eq!(meaning(Status::NeedsInput, b"\r"), Some(KeyMeaning::Answer));
    assert_eq!(meaning(Status::NeedsInput, b"2"), Some(KeyMeaning::Answer));
}

#[test]
fn a_key_that_only_looks_like_one_means_nothing() {
    let meaning = |status, bytes: &[u8]| Copilot.key_meaning(status, bytes);

    assert_eq!(meaning(Status::NeedsInput, &[0x1b, b'1']), None, "Alt+1 is ESC then 1, and not an answer");
    assert_eq!(meaning(Status::NeedsInput, b"0"), None);
    assert_eq!(meaning(Status::Working, b"\r"), None, "a queued message is not an answer to anything");
    assert_eq!(meaning(Status::Done, &[0x1b]), None, "there is no turn to stop");
    assert_eq!(meaning(Status::Idle, b"1"), None);
}

#[test]
fn the_manifest_names_the_hooks_file_beside_it() {
    let json = plugin_json();
    assert!(json.contains(r#""name": "atrium""#), "{json}");
    assert!(json.contains(r#""hooks": "hooks.json""#), "{json}");
}

#[test]
fn installing_writes_both_files_and_says_where() {
    let dir = tempfile::tempdir().expect("tempdir");
    let plugin = dir.path().join("copilot-plugin");

    let written = install_to(&plugin, "/usr/bin/atrium").expect("install");

    assert_eq!(written, plugin);
    assert_eq!(std::fs::read_to_string(plugin.join("plugin.json")).expect("manifest"), plugin_json());
    assert_eq!(std::fs::read_to_string(plugin.join("hooks.json")).expect("hooks"), hooks_json("/usr/bin/atrium"));
}

#[test]
fn the_plugin_lives_with_atriums_own_files() {
    // Nothing is written where a copilot started outside atrium would read it.
    let path = plugin_dir();
    assert!(path.ends_with("atrium/copilot-plugin"), "{}", path.display());
}
