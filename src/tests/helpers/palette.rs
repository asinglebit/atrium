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

#[test]
fn a_cell_with_no_background_of_its_own_takes_the_themes() {
    let theme = Theme::classic();
    let area = Rect::new(0, 0, 3, 2);
    let mut buffer = Buffer::empty(area);

    theme.fill_default_background(area, &mut buffer);

    for x in 0..3 {
        for y in 0..2 {
            assert_eq!(buffer[(x, y)].bg, theme.background_color(), "cell ({x}, {y}) kept the terminal's background");
        }
    }
}

#[test]
fn a_background_the_agent_set_on_purpose_survives() {
    let theme = Theme::classic();
    let area = Rect::new(0, 0, 2, 1);
    let mut buffer = Buffer::empty(area);
    buffer[(0, 0)].set_bg(Color::Rgb(1, 2, 3));

    theme.fill_default_background(area, &mut buffer);

    assert_eq!(buffer[(0, 0)].bg, Color::Rgb(1, 2, 3), "an explicit colour is not the terminal's default");
    assert_eq!(buffer[(1, 0)].bg, theme.background_color());
}

#[test]
fn filling_past_the_buffer_is_clipped_rather_than_a_panic() {
    let theme = Theme::classic();
    let mut buffer = Buffer::empty(Rect::new(0, 0, 2, 2));

    theme.fill_default_background(Rect::new(0, 0, 100, 100), &mut buffer);

    assert_eq!(buffer[(1, 1)].bg, theme.background_color());
}

#[test]
fn foregrounds_are_left_to_the_agent() {
    let theme = Theme::classic();
    let area = Rect::new(0, 0, 1, 1);
    let mut buffer = Buffer::empty(area);

    theme.fill_default_background(area, &mut buffer);

    assert_eq!(buffer[(0, 0)].fg, Color::Reset, "only the background is the theme's to set");
}
