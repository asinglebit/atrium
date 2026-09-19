use super::*;

#[test]
fn hex_colours_parse() {
    assert_eq!(parse_color("#232326"), Some(Color::Rgb(0x23, 0x23, 0x26)));
    assert_eq!(parse_color("  #ffffff  "), Some(Color::Rgb(255, 255, 255)));
}

#[test]
fn malformed_hex_is_rejected_rather_than_half_read() {
    assert_eq!(parse_color("#fff"), None, "three digits is not supported, so say so");
    assert_eq!(parse_color("#gggggg"), None);
    assert_eq!(parse_color("#1234567"), None);
}

#[test]
fn colour_names_parse_in_either_spelling() {
    assert_eq!(parse_color("red"), Some(Color::Red));
    assert_eq!(parse_color("RED"), Some(Color::Red));
    assert_eq!(parse_color("grey"), Some(Color::Gray));
    assert_eq!(parse_color("gray"), Some(Color::Gray));
    assert_eq!(parse_color("dark-grey"), Some(Color::DarkGray));
    assert_eq!(parse_color("dark_gray"), Some(Color::DarkGray));
}

#[test]
fn an_unknown_name_is_not_guessed_at() {
    assert_eq!(parse_color("burgundy"), None);
}

#[test]
fn every_advertised_preset_exists() {
    for name in PRESETS {
        assert!(Theme::preset(name).is_some(), "{name} is advertised but missing");
    }
    assert!(Theme::preset("nonsense").is_none());
}

#[test]
fn the_default_is_the_greyscale_one() {
    assert_eq!(Theme::default(), Theme::greyscale());
}

#[test]
fn each_status_gets_its_own_colour() {
    let theme = Theme::classic();
    assert_eq!(theme.for_status(Status::Working), theme.working);
    assert_eq!(theme.for_status(Status::NeedsInput), theme.needs_input);
    assert_eq!(theme.for_status(Status::Error), theme.error);
    assert_eq!(theme.for_status(Status::Exited), theme.exited);
    assert_eq!(theme.for_status(Status::Idle), theme.idle);
}

#[test]
fn setting_a_known_role_takes_and_an_unknown_one_does_not() {
    let mut theme = Theme::default();
    assert!(theme.set("border", Color::Red));
    assert_eq!(theme.border, Color::Red);
    assert!(!theme.set("bordre", Color::Red), "a typo should be reported, not silently applied");
}
