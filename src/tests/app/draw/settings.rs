use super::*;
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

/// Draws into a test terminal and hands back what it painted. Takes the
/// settings by reference so the caller keeps the row map the draw left behind,
/// which is what a click is measured against.
fn rendered(settings: &mut Settings, width: u16, height: u16) -> String {
    let keymap = Keymap::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), settings, &keymap, &Theme::classic())).expect("draw");
    terminal.backend().buffer().content().chunks(width as usize).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

/// Tall enough that nothing has to scroll out of the way.
const TALL: u16 = 40;

#[test]
fn both_tabs_are_offered() {
    let out = rendered(&mut Settings::new(&Theme::classic()), 70, TALL);
    assert!(out.contains("shortcuts"), "{out}");
    assert!(out.contains("themes"), "{out}");
}

#[test]
fn it_says_how_to_use_itself() {
    assert!(rendered(&mut Settings::new(&Theme::classic()), 70, TALL).contains("tab switches"));
}

#[test]
fn the_logo_sits_above_the_rest() {
    let out = rendered(&mut Settings::new(&Theme::classic()), 70, TALL);
    let lines: Vec<&str> = out.lines().collect();

    let logo = lines.iter().position(|line| line.contains('▀')).expect("the block should be drawn");
    let tabs = lines.iter().position(|line| line.contains("shortcuts")).expect("tab bar");
    assert!(logo < tabs, "the logo belongs above the tab bar:\n{out}");
}

#[test]
fn the_logo_is_drawn_in_atriums_own_purple() {
    let settings = &mut Settings::new(&Theme::classic());
    let keymap = Keymap::default();
    let mut terminal = Terminal::new(TestBackend::new(70, TALL)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), settings, &keymap, &Theme::classic())).expect("draw");

    let theme = Theme::classic();
    let content = terminal.backend().buffer().content();
    let painted = |colour| content.iter().any(|cell| cell.symbol() == "█" && cell.fg == colour);

    assert!(painted(theme.COLOR_PURPLE), "the block should be purple, not the text colour");
    assert!(painted(theme.COLOR_DURPLE), "the lower rows take the deeper purple, the way guitar splits its own logo");
}

#[test]
fn a_column_too_narrow_for_the_block_still_names_the_tool() {
    // 8 columns of margin come off before the logo is measured.
    let out = rendered(&mut Settings::new(&Theme::classic()), (logo::WIDTH as u16) + 4, TALL);

    assert!(out.contains(logo::COMPACT), "the word should stand in for the block:\n{out}");
    assert!(!out.contains('▀'), "the block does not fit and should not be drawn:\n{out}");
}

#[test]
fn the_shortcuts_tab_lists_every_action_with_its_chord() {
    let out = rendered(&mut Settings::new(&Theme::classic()), 70, TALL);
    for (action, chord) in Keymap::default().actions() {
        assert!(out.contains(action), "{action} missing in:\n{out}");
        assert!(out.contains(&chord.label()), "{} missing in:\n{out}", chord.label());
    }
}

#[test]
fn the_themes_tab_lists_themes_and_marks_the_one_in_use() {
    let mut settings = Settings::new(&Theme::classic());
    settings.next_tab();

    let out = rendered(&mut settings, 70, TALL);
    assert!(out.contains("classic"), "{out}");
    assert!(out.contains("in use"), "the active theme should be marked:\n{out}");
}

#[test]
fn a_section_is_named_above_its_rows() {
    let out = rendered(&mut Settings::new(&Theme::classic()), 70, TALL);
    let lines: Vec<&str> = out.lines().collect();

    let heading = lines.iter().position(|line| line.contains("keys")).expect("a named section");
    let first = lines.iter().position(|line| line.contains("new")).expect("a row");
    assert!(heading < first, "the heading belongs above its rows:\n{out}");
}

#[test]
fn rows_are_filled_so_the_eye_can_follow_across() {
    assert!(rendered(&mut Settings::new(&Theme::classic()), 70, TALL).contains('·'));
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    rendered(&mut Settings::new(&Theme::classic()), 14, 3);
}

#[test]
fn a_click_lands_on_the_row_under_it() {
    let mut settings = Settings::new(&Theme::classic());
    let area = Rect::new(0, 0, 70, TALL);
    rendered(&mut settings, 70, TALL);

    let first = settings.row_lines[0] as u16;
    assert_eq!(row_at(&settings, area, first), Some(0));
    assert_eq!(row_at(&settings, area, first + 1), Some(1));
    assert_eq!(row_at(&settings, area, first - 1), None, "the blank above the rows is not one");
}

#[test]
fn a_click_accounts_for_how_far_the_view_is_scrolled() {
    let mut settings = Settings::new(&Theme::classic());
    let area = Rect::new(0, 0, 70, TALL);
    rendered(&mut settings, 70, TALL);

    let first = settings.row_lines[0];
    settings.scroll = first;
    assert_eq!(row_at(&settings, area, 0), Some(0), "the first row is at the top once scrolled onto it");
}

#[test]
fn a_click_on_the_tab_bar_picks_that_tab() {
    let mut settings = Settings::new(&Theme::classic());
    let area = Rect::new(0, 0, 70, TALL);
    let out = rendered(&mut settings, 70, TALL);

    let row = settings.tab_line as u16;
    let line = out.lines().nth(settings.tab_line).expect("the tab bar was drawn");
    let shortcuts = line.find("shortcuts").expect("a shortcuts label") as u16;
    let themes = line.find("themes").expect("a themes label") as u16;

    assert_eq!(tab_at(&settings, area, shortcuts, row), Some(Tab::Shortcuts));
    assert_eq!(tab_at(&settings, area, themes, row), Some(Tab::Themes));
}

#[test]
fn a_click_off_the_tab_bar_picks_none_of_them() {
    let mut settings = Settings::new(&Theme::classic());
    let area = Rect::new(0, 0, 70, TALL);
    rendered(&mut settings, 70, TALL);

    let row = settings.tab_line as u16;
    assert_eq!(tab_at(&settings, area, 69, row), None, "past the labels is no tab");
    assert_eq!(tab_at(&settings, area, 3, row + 1), None, "only the bar's own line holds tabs");
}
