use super::*;

#[test]
fn every_registered_event_means_something() {
    use crate::core::agent::Status;
    for (event, atrium_event) in HOOK_EVENTS {
        // Either it names a status, or it lifts a wait. An event that does
        // neither is a hook invocation nobody reads.
        let names_a_status = Status::from_hook_event(atrium_event).is_some();
        let lifts_a_wait = Status::NeedsInput.after(atrium_event) != Status::NeedsInput;
        assert!(names_a_status || lifts_a_wait, "{event} translates to {atrium_event}, which means nothing");
    }
}

#[test]
fn every_status_a_row_can_show_has_an_event_that_reaches_it() {
    use crate::core::agent::Status;
    // The whole point of wiring copilot's hooks up: a row that can only ever
    // read idle would be no better than watching the process.
    for status in [Status::Idle, Status::Working, Status::NeedsInput, Status::Error, Status::Exited] {
        assert!(HOOK_EVENTS.iter().any(|(_, atrium_event)| Status::from_hook_event(atrium_event) == Some(status)), "nothing copilot reports means {status:?}");
    }
}

#[test]
fn the_hooks_name_every_registered_event() {
    let json = hooks_json("/usr/bin/atrium");
    for (event, atrium_event) in HOOK_EVENTS {
        assert!(json.contains(&format!(r#""{event}":"#)), "{event} missing from the hooks");
        assert!(json.contains(&format!("hook {atrium_event}")), "{event} not passed to the handler");
    }
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
    assert_eq!(json.matches(r#""type": "command""#).count(), HOOK_EVENTS.len());
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
