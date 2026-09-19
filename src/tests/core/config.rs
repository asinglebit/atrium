use super::*;
use crate::app::input::keymap::Chord;
use crossterm::event::KeyCode;

#[test]
fn an_empty_file_is_the_defaults() {
    let config = Config::parse("");
    assert_eq!(config.keymap, Keymap::default());
    assert_eq!(config.theme.name, Theme::classic().name);
    assert!(config.problems.is_empty());
}

#[test]
fn a_preset_can_be_chosen_by_name() {
    let config = Config::parse("[theme]\nname = \"ansi\"");
    assert_eq!(config.theme.name, Theme::ansi().name);
    assert!(config.problems.is_empty(), "{:?}", config.problems);
}

#[test]
fn the_base_theme_survives_a_file_that_does_not_mention_one() {
    // theme.json is the base, so an unrelated config.toml must not clobber it.
    let config = Config::parse_with("[keys]\nquit = \"z\"", Theme::matrix());
    assert_eq!(config.theme.name, Theme::matrix().name);
}

#[test]
fn a_named_preset_overrides_the_base() {
    let config = Config::parse_with("[theme]\nname = \"ansi\"", Theme::matrix());
    assert_eq!(config.theme.name, Theme::ansi().name);
}

#[test]
fn individual_colours_are_directed_to_theme_json() {
    let config = Config::parse("[theme]\nname = \"ansi\"\nborder = \"#010203\"");
    assert_eq!(config.theme.name, Theme::ansi().name, "the preset should still apply");
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("theme.json"), "{:?}", config.problems);
}

#[test]
fn keys_can_be_rebound() {
    let config = Config::parse("[keys]\nleader = \"f1\"\nquit = \"esc\"");
    assert_eq!(config.keymap.leader, Chord::plain(KeyCode::F(1)));
    assert_eq!(config.keymap.quit, Chord::plain(KeyCode::Esc));
    assert!(config.problems.is_empty(), "{:?}", config.problems);
}

#[test]
fn a_leader_can_carry_modifiers() {
    let config = Config::parse("[keys]\nleader = \"ctrl+g\"");
    assert_eq!(config.keymap.leader.label(), "ctrl+g");
    assert!(config.problems.is_empty(), "{:?}", config.problems);
}

#[test]
fn a_bare_modifier_is_reported_rather_than_silently_doing_nothing() {
    let config = Config::parse("[keys]\nleader = \"ctrl\"");
    assert_eq!(config.keymap.leader, Keymap::default().leader, "the default should be kept");
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("keys.leader"), "{:?}", config.problems);
}

#[test]
fn an_unknown_theme_is_reported_and_the_base_kept() {
    let config = Config::parse("[theme]\nname = \"burgundy\"");
    assert_eq!(config.theme.name, Theme::classic().name);
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("theme.name"), "{:?}", config.problems);
}

#[test]
fn a_misspelled_action_is_reported() {
    let config = Config::parse("[keys]\nqit = \"x\"");
    assert_eq!(config.problems.len(), 1, "{:?}", config.problems);
    assert!(config.problems[0].contains("keys.qit"));
}

#[test]
fn a_value_of_the_wrong_type_is_reported_rather_than_ignored() {
    let config = Config::parse("[keys]\nquit = 7");
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("keys.quit"));
}

#[test]
fn a_file_that_is_not_toml_at_all_keeps_the_defaults() {
    let config = Config::parse("this is not { toml");
    assert_eq!(config.theme.name, Theme::classic().name);
    assert_eq!(config.keymap, Keymap::default());
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("not valid TOML"));
}

#[test]
fn unrelated_sections_are_left_alone() {
    let config = Config::parse("[something-else]\nwhatever = true");
    assert!(config.problems.is_empty(), "atrium should not complain about a section it does not own");
}
