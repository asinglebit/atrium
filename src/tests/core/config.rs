use super::*;
use crossterm::event::KeyCode;
use ratatui::style::Color;

#[test]
fn an_empty_file_is_the_defaults() {
    let config = Config::parse("");
    assert_eq!(config, Config::default());
    assert!(config.problems.is_empty());
}

#[test]
fn a_preset_can_be_chosen_by_name() {
    let config = Config::parse(
        r##"[theme]
name = "ansi""##,
    );
    assert_eq!(config.theme, Theme::ansi());
    assert!(config.problems.is_empty());
}

#[test]
fn individual_colours_override_the_preset_whatever_the_order() {
    // "border" sorts before "name", so this only works if the preset is applied
    // first regardless of where it appears.
    let config = Config::parse(
        r##"[theme]
border = "#010203"
name = "ansi""##,
    );
    assert_eq!(config.theme.border, Color::Rgb(1, 2, 3));
    assert_eq!(config.theme.working, Theme::ansi().working, "the rest should still come from the preset");
}

#[test]
fn keys_can_be_rebound() {
    let config = Config::parse(
        r##"[keys]
leader = "f1"
quit = "esc""##,
    );
    assert_eq!(config.keymap.leader, KeyCode::F(1));
    assert_eq!(config.keymap.quit, KeyCode::Esc);
    assert!(config.problems.is_empty());
}

#[test]
fn an_unknown_theme_is_reported_and_the_default_kept() {
    let config = Config::parse(
        r##"[theme]
name = "burgundy""##,
    );
    assert_eq!(config.theme, Theme::default());
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("theme.name"), "{:?}", config.problems);
}

#[test]
fn a_bad_colour_is_reported_and_the_rest_still_applies() {
    let config = Config::parse(
        r##"[theme]
border = "not-a-colour"
error = "#ff0000""##,
    );
    assert_eq!(config.theme.error, Color::Rgb(255, 0, 0));
    assert_eq!(config.theme.border, Theme::default().border);
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("theme.border"));
}

#[test]
fn a_misspelled_role_or_action_is_reported() {
    let config = Config::parse(
        r##"[theme]
bordre = "red"

[keys]
qit = "x""##,
    );
    assert_eq!(config.problems.len(), 2, "{:?}", config.problems);
}

#[test]
fn a_value_of_the_wrong_type_is_reported_rather_than_ignored() {
    let config = Config::parse(
        r##"[keys]
quit = 7"##,
    );
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("keys.quit"));
}

#[test]
fn a_file_that_is_not_toml_at_all_keeps_the_defaults() {
    let config = Config::parse("this is not { toml");
    assert_eq!(config.theme, Theme::default());
    assert_eq!(config.keymap, Keymap::default());
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("not valid TOML"));
}

#[test]
fn unrelated_sections_are_left_alone() {
    let config = Config::parse(
        r##"[something-else]
whatever = true"##,
    );
    assert!(config.problems.is_empty(), "atrium should not complain about a section it does not own");
}
