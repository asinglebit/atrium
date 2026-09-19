use super::*;

#[test]
fn it_opens_on_the_default_so_enter_is_enough() {
    assert_eq!(Splash::new(3, 2).selected(), 2);
}

#[test]
fn a_default_out_of_range_falls_back_to_the_first() {
    assert_eq!(Splash::new(3, 99).selected(), 0);
}

#[test]
fn the_cursor_wraps_in_both_directions() {
    let mut splash = Splash::new(3, 0);

    splash.move_up();
    assert_eq!(splash.selected(), 2);

    splash.move_down();
    assert_eq!(splash.selected(), 0);
}

#[test]
fn an_empty_list_does_not_move_or_divide_by_zero() {
    let mut splash = Splash::new(0, 0);

    splash.move_down();
    splash.move_up();

    assert!(splash.is_empty());
    assert_eq!(splash.selected(), 0);
}

#[test]
fn moving_clears_a_failed_launch() {
    let mut splash = Splash::new(2, 0);
    splash.error = Some("no such program".to_owned());

    splash.move_down();

    assert!(splash.error.is_none(), "the message belongs to the choice that failed");
}

#[test]
fn a_click_lands_on_the_row_under_it() {
    let splash = Splash::new(3, 0);

    assert_eq!(splash.row_at(10, 10), Some(0));
    assert_eq!(splash.row_at(10, 12), Some(2));
}

#[test]
fn a_click_off_the_list_lands_nowhere() {
    let splash = Splash::new(3, 0);

    assert_eq!(splash.row_at(10, 9), None, "above the list");
    assert_eq!(splash.row_at(10, 13), None, "below the last row");
}
