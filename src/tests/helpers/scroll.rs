use super::*;

#[test]
fn a_list_that_fits_never_scrolls() {
    assert_eq!(trap(3, 0, 5, 10), 0);
    assert_eq!(trap(3, 7, 5, 10), 0, "a stale offset is discarded once it fits");
}

#[test]
fn scrolling_follows_the_selection_down() {
    // Ten rows, four visible: selecting row 5 must bring it into view.
    assert_eq!(trap(5, 0, 10, 4), 2);
}

#[test]
fn scrolling_follows_the_selection_up() {
    assert_eq!(trap(1, 6, 10, 4), 1);
}

#[test]
fn a_selection_already_on_screen_does_not_move_the_view() {
    assert_eq!(trap(3, 2, 10, 4), 2);
}

#[test]
fn the_offset_never_runs_past_the_end() {
    assert_eq!(trap(9, 99, 10, 4), 6, "the last row should sit at the bottom, not past it");
}

#[test]
fn a_zero_height_pane_does_not_divide_by_anything() {
    assert_eq!(trap(3, 1, 10, 0), 0);
}

#[test]
fn content_length_is_zero_when_everything_fits() {
    assert_eq!(content_length(4, 10), 0);
    assert_eq!(content_length(10, 10), 0);
}

#[test]
fn content_length_counts_the_positions_not_the_rows() {
    // Ten rows in a four-row pane can sit at seven different offsets.
    assert_eq!(content_length(10, 4), 7);
}
