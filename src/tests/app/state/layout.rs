use super::*;

#[test]
fn a_wide_frame_gets_a_sidebar() {
    let (sidebar, stage) = split(Rect::new(0, 0, 100, 30));
    let sidebar = sidebar.expect("wide frame should afford a sidebar");

    assert_eq!(sidebar.width, SIDEBAR_WIDTH);
    assert_eq!(sidebar.x, 0);
    assert_eq!(stage.x, SIDEBAR_WIDTH);
    assert_eq!(stage.width, 100 - SIDEBAR_WIDTH);
    assert_eq!(stage.height, 30);
}

#[test]
fn the_two_halves_tile_the_frame_exactly() {
    let full = Rect::new(0, 0, 120, 40);
    let (sidebar, stage) = split(full);
    let sidebar = sidebar.expect("sidebar");

    assert_eq!(sidebar.width + stage.width, full.width);
    assert_eq!(sidebar.x + sidebar.width, stage.x);
}

#[test]
fn a_narrow_frame_gives_the_whole_thing_to_the_agent() {
    let full = Rect::new(0, 0, SIDEBAR_WIDTH + MIN_STAGE_WIDTH - 1, 30);
    let (sidebar, stage) = split(full);

    assert!(sidebar.is_none(), "sidebar should be dropped rather than squeeze the agent");
    assert_eq!(stage, full);
}

#[test]
fn the_threshold_is_the_first_width_that_fits_both() {
    let full = Rect::new(0, 0, SIDEBAR_WIDTH + MIN_STAGE_WIDTH, 30);
    let (sidebar, stage) = split(full);

    assert!(sidebar.is_some());
    assert_eq!(stage.width, MIN_STAGE_WIDTH);
}

#[test]
fn an_offset_frame_keeps_its_origin() {
    let (sidebar, stage) = split(Rect::new(5, 2, 100, 30));

    assert_eq!(sidebar.expect("sidebar").x, 5);
    assert_eq!(stage.x, 5 + SIDEBAR_WIDTH);
    assert_eq!(stage.y, 2);
}
