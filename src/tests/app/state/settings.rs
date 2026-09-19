use super::*;

#[test]
fn it_opens_on_the_shortcuts_tab() {
    assert_eq!(Settings::new(&Theme::classic()).tab(), Tab::Shortcuts);
}

#[test]
fn the_themes_tab_opens_on_the_theme_in_use() {
    let mut settings = Settings::new(&Theme::matrix());
    settings.next_tab();

    assert_eq!(settings.tab(), Tab::Themes);
    assert_eq!(settings.theme_under_cursor().map(|theme| theme.name), Some(Theme::matrix().name));
}

#[test]
fn tabs_cycle_in_both_directions() {
    let mut settings = Settings::new(&Theme::classic());
    settings.previous_tab();
    assert_eq!(settings.tab(), Tab::Themes);
    settings.next_tab();
    assert_eq!(settings.tab(), Tab::Shortcuts);
}

#[test]
fn each_tab_keeps_its_own_cursor() {
    let mut settings = Settings::new(&Theme::classic());
    settings.move_down();
    settings.move_down();
    let shortcuts_at = settings.selected();

    settings.next_tab();
    settings.move_down();
    settings.previous_tab();

    assert_eq!(settings.selected(), shortcuts_at, "switching tabs should not lose your place");
}

#[test]
fn the_cursor_wraps_within_a_tab() {
    let mut settings = Settings::new(&Theme::classic());
    settings.move_up();
    assert_eq!(settings.selected(), settings.len() - 1);
    settings.move_down();
    assert_eq!(settings.selected(), 0);
}

#[test]
fn selecting_past_the_end_is_ignored() {
    let mut settings = Settings::new(&Theme::classic());
    settings.select(999);
    assert_eq!(settings.selected(), 0);
}

#[test]
fn the_shortcuts_tab_offers_no_theme() {
    assert!(Settings::new(&Theme::classic()).theme_under_cursor().is_none());
}

#[test]
fn every_theme_is_reachable_from_the_themes_tab() {
    let mut settings = Settings::new(&Theme::classic());
    settings.next_tab();
    assert_eq!(settings.len(), THEME_PRESETS.len());

    settings.select(THEME_PRESETS.len() - 1);
    assert_eq!(settings.theme_under_cursor().map(|t| t.name), Some(THEME_PRESETS[THEME_PRESETS.len() - 1].theme.name));
}
