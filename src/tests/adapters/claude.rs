use super::*;
use crate::helpers::palette::Theme;

#[test]
fn every_registered_event_means_something() {
    use crate::core::agent::Status;
    for event in HOOK_EVENTS {
        // Either it names a status, or it lifts a wait. An event that does
        // neither is a hook invocation nobody reads.
        let names_a_status = Status::from_hook_event(event).is_some();
        let lifts_a_wait = Status::NeedsInput.after(event) != Status::NeedsInput;
        assert!(names_a_status || lifts_a_wait, "{event} is registered but means nothing");
    }
}

#[test]
fn the_settings_name_every_registered_event() {
    let json = settings_json("/usr/bin/atrium", None);
    for event in HOOK_EVENTS {
        assert!(json.contains(&format!(r#""{event}":"#)), "{event} missing from settings");
        assert!(json.contains(&format!(r#"["hook","{event}"]"#)), "{event} not passed to the handler");
    }
}

#[test]
fn hooks_are_async_so_they_never_hold_the_agent_up() {
    assert_eq!(settings_json("/usr/bin/atrium", None).matches(r#""async":true"#).count(), HOOK_EVENTS.len());
}

#[test]
fn exec_form_means_a_space_in_the_path_needs_no_quoting() {
    let json = settings_json("/home/a b/atrium", None);
    assert!(json.contains(r#""command":"/home/a b/atrium","args":["hook","Stop"]"#), "got: {json}");
}

#[test]
fn a_quote_in_the_path_cannot_escape_its_json_string() {
    let json = settings_json(r#"/tmp/ev"il"#, None);
    assert!(json.contains(r#""command":"/tmp/ev\"il","args":["hook","Stop"]"#), "got: {json}");
}

#[test]
fn no_shell_is_involved_at_all() {
    let json = settings_json("/usr/bin/atrium", None);
    assert!(!json.contains(r#""shell""#), "exec form should not name a shell: {json}");
    assert_eq!(json.matches(r#""args":["hook","#).count(), HOOK_EVENTS.len());
}

#[test]
fn the_theme_rides_in_the_same_settings_document() {
    let json = settings_json("/usr/bin/atrium", Some("atrium"));

    assert!(json.contains(r#""theme":"custom:atrium""#), "{json}");
    assert!(json.contains(r#""hooks":"#), "the hooks must still be in there: {json}");
    assert_eq!(json.matches("--settings").count(), 0, "the document does not name the flag that carries it");
}

#[test]
fn a_theme_that_could_not_be_written_is_simply_not_named() {
    let json = settings_json("/usr/bin/atrium", None);

    assert!(!json.contains("theme"), "claude should be left with its own: {json}");
}

#[test]
fn the_theme_is_overrides_on_top_of_a_base() {
    let json = theme_json(&Theme::classic());

    assert!(json.contains(r#""base": "dark""#), "{json}");
    assert!(json.contains(r#""overrides""#), "{json}");
    // Keys atrium says nothing about are left to the base rather than guessed.
    assert!(!json.contains("rainbow_"), "{json}");
}

#[test]
fn a_light_theme_sits_on_the_light_base() {
    assert_eq!(base_for(&Theme::classic()), "dark");

    let mut light = Theme::classic();
    light.COLOR_GREY_950 = Color::Rgb(250, 250, 250);
    assert_eq!(base_for(&light), "light", "dark text on a light base is the readable way round");
}

#[test]
fn colours_are_written_the_way_claude_takes_them() {
    assert_eq!(value(Color::Rgb(18, 52, 86)), Some("rgb(18,52,86)".to_owned()));
    assert_eq!(value(Color::LightMagenta), Some("ansi:magentaBright".to_owned()));
    // Nothing claude can be told means "whatever the terminal had", so the key
    // is left out and the base keeps its answer.
    assert_eq!(value(Color::Reset), None);
    assert_eq!(value(Color::Indexed(238)), None);
}

#[test]
fn the_theme_lands_inside_the_subscription_it_was_launched_against() {
    let work = theme_path(Some("/home/x/.claude-work"));
    assert_eq!(work, std::path::PathBuf::from("/home/x/.claude-work/themes/atrium.json"));

    // A profile naming no directory is a claude using its own default.
    assert!(theme_path(None).ends_with(".claude/themes/atrium.json"), "{}", theme_path(None).display());
}

#[test]
fn installing_writes_the_theme_where_it_was_asked_to() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("themes").join("atrium.json");

    install_to(&path, &Theme::matrix()).expect("install");

    assert_eq!(std::fs::read_to_string(&path).expect("theme"), theme_json(&Theme::matrix()));
}
