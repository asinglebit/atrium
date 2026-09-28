use super::*;
use crate::core::agent::Status;

/// Each `-c` value the way codex reads it: as the right-hand side of a TOML
/// assignment. codex takes a value it cannot parse for a plain string, so a
/// mistake here would not fail -- it would quietly register nothing.
fn parsed(exe: &str) -> Vec<(String, toml::Value)> {
    let args = config_args(exe);
    args.chunks(2)
        .map(|pair| {
            assert_eq!(pair[0], "-c");
            let (key, value) = pair[1].split_once('=').expect("key=value");
            let table: toml::Table = format!("value = {value}").parse().unwrap_or_else(|error| panic!("{key} is not TOML: {error}\n{value}"));
            (key.to_owned(), table["value"].clone())
        })
        .collect()
}

fn command_of(value: &toml::Value) -> String {
    value[0]["hooks"][0]["command"].as_str().expect("a command").to_owned()
}

#[test]
fn every_value_is_toml_codex_can_read() {
    let values = parsed("/usr/bin/atrium");
    assert_eq!(values.len(), HOOK_EVENTS.len() + 1, "a hook per event, and the title");
}

#[test]
fn each_hook_runs_atrium_with_its_own_event() {
    for (key, value) in parsed("/usr/bin/atrium") {
        let Some(event) = key.strip_prefix("hooks.") else {
            continue;
        };
        assert_eq!(command_of(&value), format!("'/usr/bin/atrium' hook {event}"));
        assert_eq!(value[0]["hooks"][0]["type"].as_str(), Some("command"));
        assert_eq!(value[0]["hooks"][0]["timeout"].as_integer(), Some(5));
    }
}

#[test]
fn a_path_with_quotes_in_it_survives_toml_and_the_shell() {
    let exe = r#"/tmp/a "b" c'd/atrium"#;
    let values = parsed(exe);
    let (_, stop) = values.iter().find(|(key, _)| key == "hooks.Stop").expect("Stop");

    assert_eq!(command_of(stop), format!("{} hook Stop", shell::quote(exe)), "TOML gives back exactly the shell string that was meant");
}

#[test]
fn only_the_hooks_that_say_how_a_turn_began_or_ended_are_registered() {
    assert_eq!(HOOK_EVENTS, ["UserPromptSubmit", "Stop", "Interrupt"]);
    for event in HOOK_EVENTS {
        let heard = Status::from_hook_event(event).is_some() || Status::Working.after(event) != Status::Working;
        assert!(heard, "{event} would reach atrium and mean nothing");
    }
}

#[test]
fn the_title_is_asked_to_say_what_codex_is_doing() {
    let values = parsed("/usr/bin/atrium");
    let (_, title) = values.iter().find(|(key, _)| key == "tui.terminal_title").expect("the title");
    let items: Vec<&str> = title.as_array().expect("a list").iter().filter_map(toml::Value::as_str).collect();
    assert_eq!(items, ["activity", "run-state"]);
}

#[test]
fn the_title_says_what_the_hooks_do_not() {
    assert_eq!(title_activity("[ ! ] Action Required"), Some(Activity::Waiting));
    assert_eq!(title_activity("[ . ] Action Required"), Some(Activity::Waiting), "the other half of its blink");
    assert_eq!(title_activity("\u{280b} Working"), Some(Activity::Busy));
    assert_eq!(title_activity("Thinking"), Some(Activity::Busy), "with animations off there is no spinner");
    assert_eq!(title_activity("\u{2819} Waiting"), Some(Activity::Busy), "waiting on a tool, not on you");
    assert_eq!(title_activity("Ready"), Some(Activity::Idle));
    assert_eq!(title_activity("Starting"), None, "MCP servers starting is not a turn");
    assert_eq!(title_activity("~/projects/atrium"), None, "a title codex was not asked for says nothing");
}
