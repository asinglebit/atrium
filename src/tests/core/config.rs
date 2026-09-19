use super::*;
use crate::{app::input::keymap::Chord, core::profile};
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
    let config = Config::parse("[keys]\nquit = \"esc\"\nnew = \"ctrl+y\"");
    assert_eq!(config.keymap.quit, Chord::plain(KeyCode::Esc));
    assert_eq!(config.keymap.new.label(), "ctrl+y");
    assert!(config.problems.is_empty(), "{:?}", config.problems);
}

#[test]
fn an_action_can_carry_modifiers() {
    let config = Config::parse("[keys]\nnext = \"ctrl+alt+n\"");
    assert_eq!(config.keymap.next.label(), "ctrl+alt+n");
    assert!(config.problems.is_empty(), "{:?}", config.problems);
}

#[test]
fn a_leftover_leader_says_what_replaced_it() {
    let config = Config::parse("[keys]\nleader = \"f12\"");
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("no leader any more"), "{:?}", config.problems);
}

#[test]
fn a_bare_modifier_is_reported_rather_than_silently_doing_nothing() {
    let config = Config::parse("[keys]\nquit = \"ctrl\"");
    assert_eq!(config.keymap.quit, Keymap::default().quit, "the default should be kept");
    assert_eq!(config.problems.len(), 1);
    assert!(config.problems[0].contains("keys.quit"), "{:?}", config.problems);
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

/// The two subscriptions this was built for, written the way the shell aliases
/// they replace were written.
const SUBSCRIPTIONS: &str = r#"
default = "personal"

[[profiles]]
name = "work"
config_dir = "~/.claude-work"
args = ["--append-system-prompt-file", "/etc/prompt.md"]

[[profiles]]
name = "personal"
config_dir = "~/.claude-personal"
"#;

#[test]
fn a_file_with_no_profiles_offers_the_clis_atrium_knows() {
    let config = Config::parse("");

    assert_eq!(config.profiles, profile::defaults());
    assert_eq!(config.default_profile().name, "claude");
}

#[test]
fn profiles_replace_the_built_in_list_rather_than_joining_it() {
    let config = Config::parse(SUBSCRIPTIONS);

    let names: Vec<&str> = config.profiles.iter().map(|entry| entry.name.as_str()).collect();
    assert_eq!(names, ["work", "personal"]);
    assert!(config.problems.is_empty(), "{:?}", config.problems);
}

#[test]
fn a_config_dir_becomes_the_variable_that_selects_a_subscription() {
    let config = Config::parse(SUBSCRIPTIONS);
    let work = config.profile_named("work").expect("work");

    assert_eq!(work.env.first().map(|(key, _)| key.as_str()), Some(profile::CONFIG_DIR_ENV));
    assert!(work.config_dir().is_some_and(|dir| dir.ends_with("/.claude-work")), "{:?}", work.config_dir());
    assert!(!work.config_dir().unwrap().starts_with('~'), "the tilde should have been expanded");
}

#[test]
fn a_profile_launches_claude_unless_it_says_otherwise() {
    let config = Config::parse("[[profiles]]\nname = \"work\"\n\n[[profiles]]\nname = \"review\"\nprogram = \"codex\"");

    assert_eq!(config.profile_named("work").unwrap().program, "claude");
    assert_eq!(config.profile_named("review").unwrap().program, "codex");
}

#[test]
fn args_are_carried_through_in_order() {
    let config = Config::parse(SUBSCRIPTIONS);

    assert_eq!(config.profile_named("work").unwrap().args, ["--append-system-prompt-file", "/etc/prompt.md"]);
    assert!(config.profile_named("personal").unwrap().args.is_empty());
}

#[test]
fn the_default_names_which_profile_a_bare_atrium_holds() {
    let config = Config::parse(SUBSCRIPTIONS);

    assert_eq!(config.default_profile().name, "personal");
}

#[test]
fn a_default_naming_nothing_is_reported_rather_than_fatal() {
    let config = Config::parse("default = \"nope\"\n\n[[profiles]]\nname = \"work\"");

    assert_eq!(config.default_profile().name, "work", "it should still hold something");
    assert!(config.problems.iter().any(|problem| problem.contains("no profile named")), "{:?}", config.problems);
}

#[test]
fn a_profile_with_no_name_is_reported_and_skipped() {
    let config = Config::parse("[[profiles]]\nconfig_dir = \"/tmp/x\"\n\n[[profiles]]\nname = \"work\"");

    assert_eq!(config.profiles.len(), 1, "the nameless one cannot be picked, so it is not offered");
    assert!(config.problems.iter().any(|problem| problem.contains("needs a `name`")), "{:?}", config.problems);
}

#[test]
fn a_file_whose_profiles_are_all_unusable_keeps_the_built_in_list() {
    let config = Config::parse("[[profiles]]\nconfig_dir = \"/tmp/x\"");

    assert_eq!(config.profiles, profile::defaults(), "offering nothing would leave no way to hold anything");
    assert!(!config.problems.is_empty());
}

#[test]
fn a_misspelled_profile_setting_is_named() {
    let config = Config::parse("[[profiles]]\nname = \"work\"\nconfigdir = \"/tmp/x\"");

    assert!(config.problems.iter().any(|problem| problem.contains("profiles.work.configdir")), "{:?}", config.problems);
}

#[test]
fn a_setting_of_the_wrong_shape_is_reported_rather_than_ignored() {
    let config = Config::parse("[[profiles]]\nname = \"work\"\nargs = \"--flag\"\nconfig_dir = 7");

    assert!(config.problems.iter().any(|problem| problem.contains("profiles.work.args")), "{:?}", config.problems);
    assert!(config.problems.iter().any(|problem| problem.contains("profiles.work.config_dir")), "{:?}", config.problems);
    assert_eq!(config.profile_named("work").map(|entry| entry.name.as_str()), Some("work"), "the rest of it is still usable");
}

#[test]
fn an_env_table_carries_anything_a_config_dir_cannot() {
    let config = Config::parse("[[profiles]]\nname = \"work\"\nenv = { ANTHROPIC_LOG = \"debug\" }");

    assert_eq!(config.profile_named("work").unwrap().env, [("ANTHROPIC_LOG".to_owned(), "debug".to_owned())]);
}
