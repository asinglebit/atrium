use super::*;

#[test]
fn text_that_fits_is_left_alone() {
    assert_eq!(truncate_with_ellipsis("work", 10), "work");
    assert_eq!(truncate_with_ellipsis("work", 4), "work", "exactly filling is still fitting");
}

#[test]
fn text_that_does_not_fit_ends_in_three_dots() {
    assert_eq!(truncate_with_ellipsis("~/.config/atrium", 10), "~/.conf...");
    assert_eq!(truncate_with_ellipsis("~/.config/atrium", 10).chars().count(), 10, "what is cut fits the width exactly");
}

#[test]
fn a_field_too_narrow_for_an_ellipsis_is_dots_alone() {
    assert_eq!(truncate_with_ellipsis("atrium", 3), "...");
    assert_eq!(truncate_with_ellipsis("atrium", 1), ".");
    assert_eq!(truncate_with_ellipsis("atrium", 0), "");
}

#[test]
fn a_filled_row_is_exactly_the_width_asked_for() {
    let row = fill_width(" theme:", "classic ", 40);

    assert_eq!(row.chars().count(), 40);
    assert!(row.starts_with(" theme:"));
    assert!(row.ends_with("classic "));
}

#[test]
fn the_gap_takes_whatever_is_left_over() {
    assert_eq!(fill_width("ab", "yz", 8), "ab    yz");
}

#[test]
fn a_row_with_no_right_hand_side_is_padded_out() {
    let row = fill_width(" paths:", "", 20);

    assert_eq!(row.chars().count(), 20);
    assert_eq!(row.trim_end(), " paths:");
}

#[test]
fn a_row_too_full_to_fill_elides_rather_than_running_together() {
    let row = fill_width(" config:", "/a/very/long/path/that/will/not/fit", 20);

    assert_eq!(row.chars().count(), 20);
    assert!(row.ends_with("..."), "{row:?}");
}

#[test]
fn two_rows_of_one_width_line_up_however_long_their_parts_are() {
    let short = fill_width(" a:", "1", 30);
    let long = fill_width(" a much longer label:", "a much longer value", 30);

    assert_eq!(short.chars().count(), long.chars().count());
}

#[test]
fn a_multibyte_label_is_measured_in_characters_not_bytes() {
    // Three characters, six bytes. Measuring bytes would cut this short.
    let row = fill_width("日本語", "x", 10);

    assert_eq!(row.chars().count(), 10);
}
