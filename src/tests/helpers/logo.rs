use super::*;

#[test]
fn every_row_of_a_wordmark_is_the_same_width() {
    for row in WIDE {
        assert_eq!(row.chars().count(), WIDE_WIDTH, "ragged row: {row:?}");
    }
    for row in NARROW {
        assert_eq!(row.chars().count(), NARROW_WIDTH, "ragged row: {row:?}");
    }
}

#[test]
fn every_wordmark_fits_the_width_it_is_drawn_at() {
    assert!(WIDE.iter().all(|row| row.chars().count() < WIDE_COLUMNS), "the wide wordmark has to leave room around it at the width that chooses it");
    assert!(NARROW.iter().all(|row| row.chars().count() < NARROW_COLUMNS));
    assert!(COMPACT.chars().count() < NARROW_WIDTH, "the fallback has to fit where the small wordmark does not");
}

#[test]
fn the_settings_header_never_takes_the_wide_wordmark() {
    assert_eq!(rows_for(WIDE_COLUMNS).len(), NARROW.len(), "fourteen rows before the first setting is too many");
    assert_eq!(rows_for(NARROW_WIDTH).len(), NARROW.len());
}

#[test]
fn the_splash_takes_the_biggest_that_fits() {
    assert_eq!(splash_rows_for(200).len(), WIDE.len());
    assert_eq!(splash_rows_for(WIDE_COLUMNS).len(), WIDE.len());
    assert_eq!(splash_rows_for(WIDE_COLUMNS - 1).len(), NARROW.len());
    assert_eq!(splash_rows_for(NARROW_COLUMNS).len(), NARROW.len());
    assert_eq!(splash_rows_for(NARROW_COLUMNS - 1), [COMPACT]);
}

#[test]
fn a_column_too_narrow_gets_the_word_instead() {
    assert_eq!(rows_for(NARROW_WIDTH - 1), [COMPACT]);
}

#[test]
fn both_tones_reach_something_on_every_size() {
    for total in [WIDE.len(), NARROW.len()] {
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

    for rows in [WIDE.as_slice(), NARROW.as_slice()] {
        assert_eq!(tone(0, rows, &theme), theme.COLOR_PINK);
        assert_eq!(tone(rows.len() - 1, rows, &theme), theme.COLOR_PURPLE);
    }
}

#[test]
fn every_wordmark_opens_on_a_row_that_carries_ink() {
    // The share is a plain count of the rows, which only lands on the tops of
    // the letters while the top row paints something -- the dot of the `i` at
    // the least. A wordmark opening on a blank row would spend the lighter tone
    // on nothing.
    for rows in [WIDE.as_slice(), NARROW.as_slice()] {
        assert!(!rows[0].trim().is_empty(), "blank top row: {:?}", rows[0]);
    }
}

#[test]
fn the_word_ends_on_the_tail_the_drawn_ones_do() {
    assert!(COMPACT.starts_with("atriu"), "it is still the name: {COMPACT}");
    assert!(COMPACT.ends_with('ɱ'), "the hook is the tail, kept at one row: {COMPACT}");
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
