use super::*;

/// A settings state with selectable lines at the given positions, as a draw
/// would have left behind.
fn drawn_at(lines: &[usize]) -> Settings {
    let mut settings = Settings::new();
    settings.selections = lines.iter().map(|line| Selection { line: *line, kind: SelectionKind::Info }).collect();
    settings
}

#[test]
fn it_opens_on_the_first_tab() {
    assert_eq!(Settings::new().tab(), Tab::General);
}

#[test]
fn tab_walks_the_sections_and_comes_back() {
    let mut settings = Settings::new();

    for expected in [Tab::Display, Tab::Profiles, Tab::Shortcuts, Tab::General] {
        settings.next_tab();
        assert_eq!(settings.tab(), expected);
    }
}

#[test]
fn shift_tab_walks_the_other_way() {
    let mut settings = Settings::new();

    settings.previous_tab();

    assert_eq!(settings.tab(), Tab::Shortcuts);
}

#[test]
fn a_new_section_starts_at_its_top() {
    let mut settings = drawn_at(&[10, 12]);
    settings.selected = 12;
    settings.scroll = 5;

    settings.next_tab();

    assert_eq!(settings.selected, 0);
    assert_eq!(settings.scroll, 0);
    assert!(settings.selections.is_empty(), "the old section's lines do not describe the new one");
}

#[test]
fn opening_the_tab_already_open_does_not_move_the_cursor() {
    let mut settings = drawn_at(&[10]);
    settings.selected = 10;

    settings.open(Tab::General);

    assert_eq!(settings.selected, 10);
}

#[test]
fn the_cursor_snaps_onto_something_it_can_land_on() {
    let mut settings = drawn_at(&[4, 6, 8]);
    settings.selected = 0;

    settings.snap();

    assert_eq!(settings.selected, 4, "nearest, with no direction to go on");
}

#[test]
fn a_line_that_can_be_landed_on_is_left_alone() {
    let mut settings = drawn_at(&[4, 6]);
    settings.selected = 6;

    settings.snap();

    assert_eq!(settings.selected, 6);
}

#[test]
fn moving_down_onto_a_heading_carries_on_downwards() {
    let mut settings = drawn_at(&[4, 8]);
    settings.selected = 4;

    // 5, 6 and 7 are a blank, a heading and a blank.
    settings.move_down();
    settings.snap();

    assert_eq!(settings.selected, 8, "it should not have bounced back to 4");
}

#[test]
fn moving_up_onto_a_heading_carries_on_upwards() {
    let mut settings = drawn_at(&[4, 8]);
    settings.selected = 8;

    settings.move_up();
    settings.snap();

    assert_eq!(settings.selected, 4);
}

#[test]
fn walking_down_a_whole_section_never_sticks() {
    let rows = [4, 8, 9, 10, 20];
    let mut settings = drawn_at(&rows);
    settings.selected = 4;

    let mut seen = vec![4];
    for _ in 0..rows.len() {
        settings.move_down();
        settings.snap();
        seen.push(settings.selected);
    }

    assert_eq!(seen, [4, 8, 9, 10, 20, 20], "every step should advance until the end, then hold");
}

#[test]
fn the_cursor_stops_at_the_ends_rather_than_wrapping() {
    let mut settings = drawn_at(&[4, 8]);
    settings.selected = 4;

    settings.move_up();
    settings.snap();

    assert_eq!(settings.selected, 4, "nothing is above, so it stays");
}

#[test]
fn snapping_with_nothing_to_land_on_does_nothing() {
    let mut settings = Settings::new();
    settings.selected = 7;

    settings.snap();

    assert_eq!(settings.selected, 7);
}

#[test]
fn a_click_lands_only_on_a_line_that_can_be_landed_on() {
    let mut settings = drawn_at(&[4, 8]);
    settings.selected = 4;

    settings.select_line(8);
    assert_eq!(settings.selected, 8);

    settings.select_line(6);
    assert_eq!(settings.selected, 8, "a click on a heading means that heading, so it is ignored");
}

#[test]
fn the_cursor_knows_what_it_is_sitting_on() {
    let mut settings = Settings::new();
    settings.selections = vec![Selection { line: 3, kind: SelectionKind::Theme(7) }, Selection { line: 4, kind: SelectionKind::AddProfile }];
    settings.selected = 3;

    assert_eq!(settings.kind_at_cursor(), Some(&SelectionKind::Theme(7)));

    settings.selected = 5;
    assert_eq!(settings.kind_at_cursor(), None);
}

#[test]
fn a_click_on_a_tab_label_finds_that_tab() {
    let mut settings = Settings::new();
    settings.tab_hitboxes = vec![TabHitbox { tab: Tab::Display, line: 9, start: 10, end: 19 }];

    assert_eq!(settings.tab_at(9, 10), Some(Tab::Display), "the first column of the label counts");
    assert_eq!(settings.tab_at(9, 18), Some(Tab::Display));
    assert_eq!(settings.tab_at(9, 19), None, "the column after it does not");
    assert_eq!(settings.tab_at(9, 9), None);
    assert_eq!(settings.tab_at(8, 12), None, "only the bar's own line holds tabs");
}

#[test]
fn scrolling_keeps_the_cursor_on_screen() {
    let mut settings = drawn_at(&[40]);
    settings.selected = 40;

    settings.trap_scroll(60, 10);

    assert!(settings.scroll <= 40);
    assert!(settings.scroll + 10 > 40, "the selected line has to be within the visible window");
}
