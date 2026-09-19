use super::*;
use crate::core::profiles_file::StoredProfile;
use ratatui::{Terminal, backend::TestBackend};

/// Tall enough that nothing has to scroll out of the way.
const TALL: u16 = 80;

fn stored() -> StoredProfiles {
    StoredProfiles { default: "work".to_owned(), profiles: vec![StoredProfile::named("work"), StoredProfile::named("personal")] }
}

fn context<'a>(theme: &'a Theme, keymap: &'a Keymap, profiles: &'a StoredProfiles) -> Context<'a> {
    Context { keymap, theme, profiles, default_profile: 0, socket: Path::new("/run/user/1000/atrium/1.sock") }
}

/// Draws and hands back what was painted, keeping whatever the draw recorded on
/// the settings state so a click can be measured against it.
fn rendered(settings: &mut Settings, width: u16, height: u16) -> String {
    let theme = Theme::classic();
    let keymap = Keymap::default();
    let profiles = stored();
    let context = context(&theme, &keymap, &profiles);

    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), frame.area(), settings, &context)).expect("draw");
    terminal.backend().buffer().content().chunks(width as usize).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

fn on_tab(tab: Tab, width: u16) -> (Settings, String) {
    let mut settings = Settings::new();
    settings.open(tab);
    let out = rendered(&mut settings, width, TALL);
    (settings, out)
}

#[test]
fn the_column_keeps_guitars_margin_and_ceiling() {
    assert_eq!(content_width(Rect::new(0, 0, 120, 40)), 106, "it stops growing at guitar's ceiling");
    assert_eq!(content_width(Rect::new(0, 0, 80, 40)), 71);
    assert_eq!(content_width(Rect::new(0, 0, 200, 40)), MAX_CONTENT_WIDTH);
    assert_eq!(content_width(Rect::new(0, 0, 4, 40)), 0, "a pane with no room does not underflow");
}

#[test]
fn every_row_in_one_pass_is_the_same_width() {
    let (_, out) = on_tab(Tab::General, 90);
    // A heading ends at its colon; a filled row carries a value after it.
    let widths: Vec<usize> = out.lines().map(str::trim_end).filter(|line| line.contains(':') && !line.ends_with(':')).map(|line| line.chars().count()).collect();

    assert!(widths.len() > 3, "expected several rows:\n{out}");
    // Every filled row ends at the same column, which is what lines them up.
    assert!(widths.windows(2).all(|pair| pair[0] == pair[1]), "ragged rows: {widths:?}\n{out}");
}

#[test]
fn a_value_too_long_for_the_column_ends_in_three_dots() {
    let (_, out) = on_tab(Tab::General, 40);

    assert!(out.contains("..."), "a narrow column should elide a path:\n{out}");
}

#[test]
fn a_heading_sits_between_two_blank_lines() {
    let (_, out) = on_tab(Tab::General, 90);
    let lines: Vec<&str> = out.lines().collect();
    let heading = lines.iter().position(|line| line.contains("paths:")).expect("a heading");

    assert!(lines[heading - 1].trim().is_empty(), "no blank above:\n{out}");
    assert!(lines[heading + 1].trim().is_empty(), "no blank below:\n{out}");
}

#[test]
fn rows_alternate_a_shaded_background() {
    let theme = Theme::classic();
    let keymap = Keymap::default();
    let profiles = stored();
    let context = context(&theme, &keymap, &profiles);
    let mut settings = Settings::new();

    let mut terminal = Terminal::new(TestBackend::new(90, TALL)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), frame.area(), &mut settings, &context)).expect("draw");

    let buffer = terminal.backend().buffer();
    let shaded = theme.background_or_default(theme.COLOR_GREY_900);
    let rows: Vec<bool> = (0..TALL).map(|row| (0..90).any(|column| buffer[(column, row)].bg == shaded)).collect();

    assert!(rows.iter().any(|shaded| *shaded), "no row was shaded at all");
}

#[test]
fn the_version_and_the_wordmark_head_every_tab() {
    for tab in Tab::ALL {
        let (_, out) = on_tab(tab, 90);
        assert!(out.contains("version:"), "{tab:?} has no version row:\n{out}");
        assert!(out.contains('▀'), "{tab:?} has no wordmark:\n{out}");
    }
}

#[test]
fn the_tab_bar_names_every_section() {
    let (_, out) = on_tab(Tab::General, 90);

    for tab in Tab::ALL {
        assert!(out.contains(tab.label()), "{} missing from the bar:\n{out}", tab.label());
    }
}

#[test]
fn a_column_too_narrow_for_the_labels_falls_back_to_dots() {
    let (_, out) = on_tab(Tab::General, 36);

    assert!(out.contains(COMPACT_TAB), "the bar should narrow rather than overflow:\n{out}");
    assert!(!out.contains("shortcuts"), "the full labels do not fit and should not be drawn:\n{out}");
}

#[test]
fn a_click_at_a_recorded_hitbox_opens_that_tab() {
    let (settings, _) = on_tab(Tab::General, 90);

    for hitbox in settings.tab_hitboxes.clone() {
        assert_eq!(settings.tab_at(hitbox.line, hitbox.start), Some(hitbox.tab));
    }
}

#[test]
fn general_lists_the_files_atrium_reads_and_writes() {
    let (_, out) = on_tab(Tab::General, 100);

    for label in ["paths:", "config:", "profiles:", "theme:", "layout:", "projects:", "root:", "socket:"] {
        assert!(out.contains(label), "{label} missing:\n{out}");
    }
}

#[test]
fn display_lists_every_theme_with_exactly_one_marked() {
    let (_, out) = on_tab(Tab::Display, 90);

    assert!(out.contains("themes:"));
    assert!(out.contains("classic"), "{out}");
    assert_eq!(out.matches(RADIO_ON).count(), 1, "exactly one theme is in use:\n{out}");
    assert!(out.matches(RADIO_OFF).count() > 5, "the rest should be unmarked:\n{out}");
}

#[test]
fn profiles_lists_what_is_configured_and_marks_the_default() {
    let (_, out) = on_tab(Tab::Profiles, 90);

    assert!(out.contains("+ add profile"), "{out}");
    assert!(out.contains("select to manage"), "the actions hint belongs here:\n{out}");
    assert!(out.contains("work") && out.contains("personal"), "{out}");
    assert!(out.contains("~/.claude-work"), "a profile shows its directory as written:\n{out}");
    assert_eq!(out.matches(RADIO_ON).count(), 1, "exactly one default:\n{out}");
}

#[test]
fn shortcuts_lists_every_action_with_its_chord() {
    let (_, out) = on_tab(Tab::Shortcuts, 90);

    assert!(out.contains("keys:"));
    for (action, chord) in Keymap::default().actions() {
        assert!(out.contains(action), "{action} missing:\n{out}");
        assert!(out.contains(&chord.label()), "{} missing:\n{out}", chord.label());
    }
}

#[test]
fn the_cursor_lands_on_a_row_rather_than_a_heading() {
    let (settings, _) = on_tab(Tab::General, 90);

    assert!(settings.kind_at_cursor().is_some(), "the draw should have snapped it onto something");
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    rendered(&mut Settings::new(), 14, 4);
    rendered(&mut Settings::new(), 2, 2);
    let mut narrow = Settings::new();
    narrow.open(Tab::Profiles);
    rendered(&mut narrow, 10, 6);
}
