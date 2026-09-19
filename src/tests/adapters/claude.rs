use super::*;

#[test]
fn every_registered_event_maps_to_a_status() {
    use crate::core::agent::Status;
    for event in HOOK_EVENTS {
        assert!(Status::from_hook_event(event).is_some(), "{event} is registered but means nothing");
    }
}

#[test]
fn the_settings_name_every_registered_event() {
    let json = settings_json("/usr/bin/atrium");
    for event in HOOK_EVENTS {
        assert!(json.contains(&format!(r#""{event}":"#)), "{event} missing from settings");
        assert!(json.contains(&format!(r#"["hook","{event}"]"#)), "{event} not passed to the handler");
    }
}

#[test]
fn hooks_are_async_so_they_never_hold_the_agent_up() {
    assert_eq!(settings_json("/usr/bin/atrium").matches(r#""async":true"#).count(), HOOK_EVENTS.len());
}

#[test]
fn exec_form_means_a_space_in_the_path_needs_no_quoting() {
    let json = settings_json("/home/a b/atrium");
    assert!(json.contains(r#""command":"/home/a b/atrium","args":["hook","Stop"]"#), "got: {json}");
}

#[test]
fn a_quote_in_the_path_cannot_escape_its_json_string() {
    let json = settings_json(r#"/tmp/ev"il"#);
    assert!(json.contains(r#""command":"/tmp/ev\"il","args":["hook","Stop"]"#), "got: {json}");
}

#[test]
fn no_shell_is_involved_at_all() {
    let json = settings_json("/usr/bin/atrium");
    assert!(!json.contains(r#""shell""#), "exec form should not name a shell: {json}");
    assert_eq!(json.matches(r#""args":["hook","#).count(), HOOK_EVENTS.len());
}
