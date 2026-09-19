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
