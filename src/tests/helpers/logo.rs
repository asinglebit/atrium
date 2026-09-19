use super::*;

#[test]
fn every_row_of_the_block_is_the_same_width() {
    for row in WIDE {
        assert_eq!(row.chars().count(), WIDTH, "ragged row: {row:?}");
    }
}

#[test]
fn a_column_wide_enough_gets_the_block() {
    assert_eq!(rows_for(WIDTH).len(), WIDE.len());
    assert_eq!(rows_for(200).len(), WIDE.len());
}

#[test]
fn a_column_too_narrow_gets_the_word_instead() {
    let rows = rows_for(WIDTH - 1);

    assert_eq!(rows, [COMPACT]);
    assert!(COMPACT.chars().count() < WIDTH, "the fallback has to fit where the block does not");
}
