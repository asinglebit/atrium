use super::*;
use crate::core::profile::Profile;
use ratatui::{Terminal, backend::TestBackend};

fn harnesses() -> Vec<Profile> {
    vec![
        Profile { name: "work".to_owned(), program: "claude".to_owned(), args: Vec::new(), env: Vec::new() },
        Profile { name: "personal".to_owned(), program: "claude".to_owned(), args: Vec::new(), env: Vec::new() },
        Profile::bare("opencode"),
    ]
}

fn rendered(splash: &Splash, width: u16, height: u16) -> String {
    rendered_with(splash, &harnesses(), width, height)
}

fn rendered_with(splash: &Splash, profiles: &[Profile], width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), splash, profiles, &Theme::classic())).expect("draw");
    terminal.backend().buffer().content().chunks(width as usize).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

#[test]
fn it_lists_every_harness_by_its_label() {
    let out = rendered(&Splash::new(3, 0), 80, 30);

    assert!(out.contains("claude · work"), "{out}");
    assert!(out.contains("claude · personal"), "{out}");
    assert!(out.contains("opencode"), "a bare cli names itself once:\n{out}");
}

#[test]
fn the_wordmark_sits_above_the_list() {
    let out = rendered(&Splash::new(3, 0), 80, 30);
    let lines: Vec<&str> = out.lines().collect();

    let logo = lines.iter().position(|line| line.contains("X@BwP@")).expect("the wordmark:\n{out}");
    let heading = lines.iter().position(|line| line.contains(HEADING)).expect("the heading");
    let first = lines.iter().position(|line| line.contains("work")).expect("a harness");

    assert!(logo < heading && heading < first, "wordmark, heading, then the list:\n{out}");
}

#[test]
fn the_selected_row_is_the_one_in_brackets() {
    let out = rendered(&Splash::new(3, 1), 80, 30);
    let pointed = out.lines().find(|line| line.contains(SELECTED_LEFT)).expect("a marked row");

    assert!(pointed.contains("personal"), "{out}");
    assert!(pointed.contains(SELECTED_RIGHT));
}

#[test]
fn the_content_is_centred_in_the_frame() {
    let out = rendered(&Splash::new(3, 0), 80, 40);
    let filled: Vec<usize> = out.lines().enumerate().filter(|(_, line)| !line.trim().is_empty()).map(|(index, _)| index).collect();

    let above = filled[0];
    let below = 40 - 1 - filled[filled.len() - 1];
    assert!(above.abs_diff(below) <= 1, "content should sit in the middle, got {above} above and {below} below:\n{out}");
}

#[test]
fn a_wide_frame_gets_the_wordmark_at_its_largest() {
    let out = rendered(&Splash::new(3, 0), 130, 40);

    assert!(out.contains("I$#R#BR$$"), "{out}");
}

#[test]
fn a_narrower_frame_falls_back_to_the_smaller_wordmark() {
    let out = rendered(&Splash::new(3, 0), 100, 30);

    assert!(!out.contains("I$#R#BR$$"), "the wide wordmark is not what 100 columns gets:\n{out}");
    assert!(out.contains("X@BwP@"), "the small one should stand in for it:\n{out}");
}

#[test]
fn a_frame_too_narrow_for_either_still_names_the_tool() {
    let out = rendered(&Splash::new(3, 0), 60, 30);

    assert!(out.contains(logo::COMPACT), "{out}");
    assert!(!out.contains("X@BwP@"), "below 80 columns nothing is drawn but the word:\n{out}");
}

#[test]
fn with_nothing_to_hold_it_says_what_it_looked_for() {
    let out = rendered_with(&Splash::new(0, 0), &[], 80, 30);

    assert!(out.contains("nothing installed"), "an empty list on its own says nothing:\n{out}");
    for program in KNOWN_PROGRAMS {
        assert!(out.contains(program), "it should name {program}, which is what it looked for:\n{out}");
    }
}

#[test]
fn it_says_what_the_keys_do() {
    assert!(rendered(&Splash::new(3, 0), 80, 30).contains("enter holds one here"));
}

#[test]
fn a_failed_launch_is_shown_and_the_list_moves_down_with_it() {
    let mut splash = Splash::new(3, 0);
    let before = first_row(Rect::new(0, 0, 80, 30), &splash, 3);

    splash.error = Some("no such program".to_owned());
    let out = rendered(&splash, 80, 30);

    assert!(out.contains("no such program"), "{out}");
    assert_ne!(first_row(Rect::new(0, 0, 80, 30), &splash, 3), before, "the list has to account for the message above it");
}

#[test]
fn a_click_is_measured_against_where_the_list_was_drawn() {
    let splash = Splash::new(3, 0);
    let area = Rect::new(0, 0, 80, 30);
    let out = rendered(&splash, 80, 30);

    let first = first_row(area, &splash, 3);
    let drawn = out.lines().position(|line| line.contains("work")).expect("a harness row") as u16;

    assert_eq!(first, drawn, "the row a click resolves to must be the row that was painted:\n{out}");
}

#[test]
fn it_fits_a_frame_too_small_for_it() {
    rendered(&Splash::new(3, 0), 20, 4);
    rendered(&Splash::new(0, 0), 2, 2);
}

#[test]
fn the_wordmark_lightens_at_the_top() {
    let theme = Theme::classic();
    let profiles = harnesses();
    let splash = Splash::new(3, 0);

    let mut terminal = Terminal::new(TestBackend::new(80, 30)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), &splash, &profiles, &theme)).expect("draw");

    let buffer = terminal.backend().buffer();
    let rows_with = |colour| (0..30).filter(|row| (0..80).any(|column| buffer[(column, *row)].fg == colour && buffer[(column, *row)].symbol() != " ")).collect::<Vec<u16>>();

    let pink = rows_with(theme.COLOR_PINK);
    let dark = rows_with(theme.COLOR_PURPLE);

    assert!(!pink.is_empty(), "nothing was drawn in the lighter tone");
    assert!(!dark.is_empty(), "nothing was drawn in the darker one");
    assert!(pink.iter().max() < dark.iter().min(), "every pink row belongs above every dark one, got pink {pink:?} and dark {dark:?}");
}

#[test]
fn nothing_is_drawn_around_it() {
    let out = rendered(&Splash::new(3, 0), 80, 30);

    // The splash wears no chrome: the app draws no frame, title line or status
    // line while it is up, so none of their glyphs should appear.
    for glyph in ['╭', '╮', '╯', '╰', '│', '─'] {
        assert!(!out.contains(glyph), "{glyph} should not be on a bare splash:\n{out}");
    }
}
