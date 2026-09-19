use super::*;

#[test]
fn every_row_of_a_wordmark_is_the_same_width() {
    for row in WIDE {
        assert_eq!(row.chars().count(), WIDE_WIDTH, "ragged row: {row:?}");
    }
    for row in BLOCK {
        assert_eq!(row.chars().count(), BLOCK_WIDTH, "ragged row: {row:?}");
    }
}

#[test]
fn the_settings_header_never_takes_the_tall_one() {
    assert_eq!(rows_for(WIDE_WIDTH).len(), BLOCK.len(), "eleven rows before the first setting is too many");
    assert_eq!(rows_for(BLOCK_WIDTH).len(), BLOCK.len());
}

#[test]
fn the_splash_takes_the_biggest_that_fits() {
    assert_eq!(splash_rows_for(200).len(), WIDE.len());
    assert_eq!(splash_rows_for(WIDE_WIDTH).len(), WIDE.len());
    assert_eq!(splash_rows_for(WIDE_WIDTH - 1).len(), BLOCK.len());
    assert_eq!(splash_rows_for(BLOCK_WIDTH - 1), [COMPACT]);
}

#[test]
fn a_column_too_narrow_gets_the_word_instead() {
    assert_eq!(rows_for(BLOCK_WIDTH - 1), [COMPACT]);
    assert!(COMPACT.chars().count() < BLOCK_WIDTH, "the fallback has to fit where the block does not");
}

#[test]
fn both_tones_reach_something_on_every_size() {
    for total in [WIDE.len(), BLOCK.len()] {
        let bright = bright_rows(total);
        assert!(bright > 0, "{total} rows left the lighter tone with nothing");
        assert!(bright < total, "{total} rows left the darker tone with nothing");
    }
}

#[test]
fn the_lighter_tone_is_the_top_third_rounded_up() {
    assert_eq!(bright_rows(10), 3);
    assert_eq!(bright_rows(5), 2, "a short wordmark still gets some");
    assert_eq!(bright_rows(1), 1);
    assert_eq!(bright_rows(0), 0);
}

#[test]
fn the_top_rows_are_pink_and_the_rest_are_darker() {
    let theme = Theme::classic();

    assert_eq!(tone(0, &WIDE, &theme), theme.COLOR_PINK);
    assert_eq!(tone(WIDE.len() - 1, &WIDE, &theme), theme.COLOR_PURPLE);
    assert_eq!(tone(0, &BLOCK, &theme), theme.COLOR_PINK);
    assert_eq!(tone(BLOCK.len() - 1, &BLOCK, &theme), theme.COLOR_PURPLE);
}

#[test]
fn the_share_is_counted_over_the_rows_that_carry_ink() {
    let theme = Theme::classic();

    // The wide wordmark opens with a blank row. Counting it would leave the
    // lighter tone on nothing but the dot of the i and the stem of the t.
    assert!(WIDE[0].trim().is_empty(), "this test is about that blank row");
    let pink = (0..WIDE.len()).filter(|index| tone(*index, &WIDE, &theme) == theme.COLOR_PINK).count();

    assert_eq!(pink, 4, "the blank row plus the top three that carry ink");
    assert!(!WIDE[pink - 1].trim().is_empty(), "the last pink row should be one you can see");
}

#[test]
fn a_single_row_takes_the_lighter_tone() {
    let theme = Theme::classic();

    assert_eq!(tone(0, &[COMPACT], &theme), theme.COLOR_PINK, "there is no second tone to fall into");
}

#[test]
fn a_single_row_is_all_bright() {
    assert_eq!(bright_rows(1), 1, "there is no second tone to split into");
    assert_eq!(bright_rows(0), 0);
}
