use super::*;

#[test]
fn it_opens_on_the_agent_already_showing() {
    assert_eq!(Goto::new(4, 2).selected(), 2);
}

#[test]
fn an_out_of_range_focus_is_clamped() {
    assert_eq!(Goto::new(2, 9).selected(), 1);
}

#[test]
fn it_wraps_in_both_directions() {
    let mut goto = Goto::new(3, 0);
    goto.move_up();
    assert_eq!(goto.selected(), 2);
    goto.move_down();
    assert_eq!(goto.selected(), 0);
}

#[test]
fn moving_with_nothing_held_is_harmless() {
    let mut goto = Goto::new(0, 0);
    goto.move_down();
    goto.move_up();
    assert_eq!(goto.selected(), 0);
    assert!(goto.is_empty());
}

#[test]
fn digits_name_rows_from_one() {
    let goto = Goto::new(3, 0);
    assert_eq!(goto.row_for('1'), Some(0));
    assert_eq!(goto.row_for('3'), Some(2));
}

#[test]
fn zero_names_the_tenth_row() {
    assert_eq!(Goto::new(10, 0).row_for('0'), Some(9));
    assert_eq!(Goto::new(3, 0).row_for('0'), None, "there is no tenth row to name");
}

#[test]
fn a_digit_past_the_end_names_nothing() {
    let goto = Goto::new(3, 0);
    assert_eq!(goto.row_for('4'), None);
    assert_eq!(goto.row_for('9'), None);
}

#[test]
fn a_non_digit_names_nothing() {
    assert_eq!(Goto::new(3, 0).row_for('j'), None);
}
