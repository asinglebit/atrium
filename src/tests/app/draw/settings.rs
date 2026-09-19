use super::*;
use ratatui::{Terminal, backend::TestBackend, layout::Rect};

fn rendered(settings: &Settings, width: u16, height: u16) -> String {
    let keymap = Keymap::default();
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), settings, &keymap, &Theme::classic())).expect("draw");
    terminal.backend().buffer().content().chunks(width as usize).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

#[test]
fn both_tabs_are_offered() {
    let out = rendered(&Settings::new(&Theme::classic()), 70, 14);
    assert!(out.contains("shortcuts"), "{out}");
    assert!(out.contains("themes"), "{out}");
}

#[test]
fn it_says_how_to_use_itself() {
    assert!(rendered(&Settings::new(&Theme::classic()), 70, 14).contains("tab switches"));
}

#[test]
fn the_shortcuts_tab_lists_every_action_with_its_chord() {
    let out = rendered(&Settings::new(&Theme::classic()), 70, 16);
    for (action, chord) in Keymap::default().actions() {
        assert!(out.contains(action), "{action} missing in:\n{out}");
        assert!(out.contains(&chord.label()), "{} missing in:\n{out}", chord.label());
    }
}

#[test]
fn the_themes_tab_lists_themes_and_marks_the_one_in_use() {
    let mut settings = Settings::new(&Theme::classic());
    settings.next_tab();

    let out = rendered(&settings, 70, 16);
    assert!(out.contains("classic"), "{out}");
    assert!(out.contains("in use"), "the active theme should be marked:\n{out}");
}

#[test]
fn rows_are_filled_so_the_eye_can_follow_across() {
    assert!(rendered(&Settings::new(&Theme::classic()), 70, 14).contains('·'));
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    let settings = Settings::new(&Theme::classic());
    let keymap = Keymap::default();
    let mut terminal = Terminal::new(TestBackend::new(14, 3)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), &settings, &keymap, &Theme::classic())).expect("should not panic on a small frame");
}

#[test]
fn a_click_lands_on_the_row_under_it() {
    let area = Rect::new(0, 2, 70, 12);
    // The tab header takes the first two rows.
    assert_eq!(row_at(area, 0, 2), None);
    assert_eq!(row_at(area, 0, 3), None);
    assert_eq!(row_at(area, 0, 4), Some(0));
    assert_eq!(row_at(area, 5, 4), Some(5));
}

#[test]
fn a_click_on_the_tab_row_picks_that_tab() {
    let area = Rect::new(0, 2, 70, 12);
    // " shortcuts " starts at column 1; " themes " follows it.
    assert_eq!(tab_at(area, 3, 2), Some(Tab::Shortcuts));
    assert_eq!(tab_at(area, 16, 2), Some(Tab::Themes));
}

#[test]
fn a_click_past_the_tabs_picks_none_of_them() {
    let area = Rect::new(0, 2, 70, 12);
    assert_eq!(tab_at(area, 60, 2), None);
    assert_eq!(tab_at(area, 3, 5), None, "only the header row holds tabs");
}
