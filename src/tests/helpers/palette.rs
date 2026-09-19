use super::*;
use crate::core::agent::Status;

#[test]
fn every_preset_can_be_found_by_its_label() {
    for preset in THEME_PRESETS {
        assert!(preset_named(preset.label).is_some(), "{} is listed but not findable", preset.label);
    }
}

#[test]
fn an_unknown_label_is_not_guessed_at() {
    assert!(preset_named("burgundy").is_none());
}

#[test]
fn there_are_as_many_themes_as_guitar_has() {
    // Copied wholesale from guitar; if this drops, the copy has been trimmed.
    assert!(THEME_PRESETS.len() >= 30, "only {} presets", THEME_PRESETS.len());
}

#[test]
fn the_background_stands_in_for_reset() {
    let theme = Theme::ansi();
    assert_eq!(theme.background_or_default(Color::Reset), theme.background_color());
}

#[test]
fn a_real_colour_is_left_alone() {
    let theme = Theme::classic();
    assert_eq!(theme.background_or_default(Color::Red), Color::Red);
    assert_eq!(theme.background_or_default(theme.COLOR_GREY_900), theme.COLOR_GREY_900);
}

#[test]
fn every_status_maps_onto_a_palette_colour() {
    use crate::app::draw::pane::status_color;
    let theme = Theme::classic();

    assert_eq!(status_color(&theme, Status::Error), theme.COLOR_RED);
    assert_eq!(status_color(&theme, Status::NeedsInput), theme.COLOR_GREEN);
    assert_eq!(status_color(&theme, Status::Working), theme.COLOR_AMBER);
    assert_eq!(status_color(&theme, Status::Idle), theme.COLOR_GREY_400);
    assert_eq!(status_color(&theme, Status::Exited), theme.COLOR_GREY_600);
}

#[test]
fn distinct_statuses_are_told_apart_by_colour() {
    use crate::app::draw::pane::status_color;
    let theme = Theme::classic();
    let colours = [Status::Idle, Status::Working, Status::NeedsInput, Status::Error].map(|status| status_color(&theme, status));

    for (index, colour) in colours.iter().enumerate() {
        assert!(!colours[index + 1..].contains(colour), "two statuses share {colour:?}");
    }
}
