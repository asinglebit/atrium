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
    use crate::app::draw::pane::status_style;
    let theme = Theme::classic();

    assert_eq!(status_style(&theme, Status::Error, true).fg, Some(theme.COLOR_RED));
    assert_eq!(status_style(&theme, Status::NeedsInput, true).fg, Some(theme.COLOR_BLUE));
    assert_eq!(status_style(&theme, Status::Working, true).fg, Some(theme.COLOR_ORANGE));
    assert_eq!(status_style(&theme, Status::Idle, true).fg, Some(theme.COLOR_GREEN));
    assert_eq!(status_style(&theme, Status::Exited, true).fg, Some(theme.COLOR_GREY_600));
}

#[test]
fn only_a_status_still_waiting_pulses() {
    use crate::app::draw::pane::status_style;
    let theme = Theme::classic();
    let dark = |status| status_style(&theme, status, false).fg;

    assert_eq!(dark(Status::NeedsInput), Some(theme.COLOR_GREY_600), "an agent waiting on you has to be noticed");
    assert_eq!(dark(Status::Working), Some(theme.COLOR_GREY_600));
    assert_eq!(dark(Status::Idle), Some(theme.COLOR_GREEN), "a finished agent is settled");
    assert_eq!(dark(Status::Error), Some(theme.COLOR_RED));
    assert_eq!(dark(Status::Exited), Some(theme.COLOR_GREY_600));
}

#[test]
fn nothing_asks_the_terminal_to_blink() {
    // ghostty parses SGR 5 and ignores it, so a modifier here would be a pulse
    // that never happens.
    use crate::app::draw::pane::status_style;
    use ratatui::style::Modifier;
    let theme = Theme::classic();

    for status in [Status::Idle, Status::Working, Status::NeedsInput, Status::Error, Status::Exited] {
        for lit in [true, false] {
            let modifiers = status_style(&theme, status, lit).add_modifier;
            assert!(!modifiers.contains(Modifier::SLOW_BLINK) && !modifiers.contains(Modifier::RAPID_BLINK), "{status:?} asks the terminal to blink");
        }
    }
}

#[test]
fn distinct_statuses_are_told_apart_by_colour() {
    use crate::app::draw::pane::status_style;
    let theme = Theme::classic();
    let colours = [Status::Idle, Status::Working, Status::NeedsInput, Status::Error].map(|status| status_style(&theme, status, true).fg);

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

/// How far apart two colours read, on the ratio the accessibility guidelines
/// use. 4.5 is their bar for ordinary text.
fn contrast(left: Color, right: Color) -> f32 {
    let channel = |c: u8| {
        let c = f32::from(c) / 255.0;
        if c <= 0.03928 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }
    };
    let luminance = |colour: Color| match colour {
        Color::Rgb(red, green, blue) => Some(0.2126 * channel(red) + 0.7152 * channel(green) + 0.0722 * channel(blue)),
        _ => None,
    };
    let (Some(left), Some(right)) = (luminance(left), luminance(right)) else {
        return f32::MAX;
    };
    (left.max(right) + 0.05) / (left.min(right) + 0.05)
}

#[test]
fn a_lifted_colour_clears_the_background_it_is_lifted_from() {
    // The splash's selected row is the pink lifted this far. Half the themes are
    // light and half dark, and the green it replaced washed out on twelve of
    // them -- lifting away from the background rather than always toward white
    // is what carries all sixty.
    for preset in THEME_PRESETS {
        let theme = preset.theme;
        let background = theme.background_color();
        let lifted = lift(theme.COLOR_PINK, background, 0.35);
        if !matches!((lifted, background), (Color::Rgb(..), Color::Rgb(..))) {
            continue;
        }
        assert!(contrast(lifted, background) >= 4.5, "{} reads the selected row at {:.1}, which is under the bar", preset.label, contrast(lifted, background));
    }
}

#[test]
fn a_lift_goes_away_from_the_background_whichever_end_it_is_at() {
    let pink = Color::Rgb(236, 64, 122);
    let brightness = |colour| match colour {
        Color::Rgb(red, green, blue) => 0.299 * f32::from(red) + 0.587 * f32::from(green) + 0.114 * f32::from(blue),
        _ => 0.0,
    };

    let over_dark = lift(pink, Color::Rgb(18, 18, 18), 0.35);
    let over_pale = lift(pink, Color::Rgb(250, 244, 237), 0.35);

    assert!(brightness(over_dark) > brightness(pink), "a dark theme has room upward");
    assert!(brightness(over_pale) < brightness(pink), "a pale one has it downward");
}

#[test]
fn a_theme_with_nothing_to_mix_keeps_its_colour() {
    assert_eq!(lift(Color::Magenta, Color::Black, 0.35), Color::Magenta);
}
