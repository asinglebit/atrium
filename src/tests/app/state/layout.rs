use super::*;

#[test]
fn the_frame_sits_between_a_title_line_and_a_status_line() {
    let layout = compute(Rect::new(0, 0, 100, 30));

    assert_eq!(layout.title_left.y, 0);
    assert_eq!(layout.title_left.height, 1);
    assert_eq!(layout.app.y, 1);
    assert_eq!(layout.statusbar_left.y, 29);
    assert_eq!(layout.app.y + layout.app.height, layout.statusbar_left.y, "the frame should reach the status line");
}

#[test]
fn the_title_and_status_lines_are_split_into_halves_that_tile() {
    let layout = compute(Rect::new(0, 0, 100, 30));

    assert_eq!(layout.title_left.width + layout.title_right.width, 100);
    assert_eq!(layout.title_left.x + layout.title_left.width, layout.title_right.x);
    assert_eq!(layout.statusbar_left.width + layout.statusbar_right.width, 100);
}

#[test]
fn the_panes_sit_inside_the_frames_border() {
    let layout = compute(Rect::new(0, 0, 100, 30));
    let sidebar = layout.sidebar.expect("wide frame should afford a sidebar");

    assert_eq!(sidebar.x, layout.app.x + 1, "a pane must not sit on the frame's border");
    assert_eq!(sidebar.y, layout.app.y + 1);
    assert_eq!(layout.stage.x + layout.stage.width, layout.app.x + layout.app.width - 1);
    assert_eq!(sidebar.height, layout.app.height - 2);
}

#[test]
fn the_sidebar_and_stage_tile_the_space_inside_the_frame() {
    let layout = compute(Rect::new(0, 0, 100, 30));
    let sidebar = layout.sidebar.expect("sidebar");

    assert_eq!(sidebar.width, SIDEBAR_WIDTH);
    assert_eq!(sidebar.x + sidebar.width, layout.stage.x);
    assert_eq!(sidebar.width + layout.stage.width, layout.app.width - 2);
}

#[test]
fn a_narrow_frame_gives_the_whole_inside_to_the_agent() {
    let layout = compute(Rect::new(0, 0, SIDEBAR_WIDTH + MIN_STAGE_WIDTH, 30));

    assert!(layout.sidebar.is_none(), "sidebar should be dropped rather than squeeze the agent");
    assert_eq!(layout.stage.width, layout.app.width - 2);
}

#[test]
fn the_threshold_is_the_first_inside_width_that_fits_both() {
    // Two more columns than the panes need, because the frame's border takes them.
    let layout = compute(Rect::new(0, 0, SIDEBAR_WIDTH + MIN_STAGE_WIDTH + 2, 30));

    assert!(layout.sidebar.is_some());
    assert_eq!(layout.stage.width, MIN_STAGE_WIDTH);
}

#[test]
fn a_frame_too_short_for_chrome_still_produces_something_drawable() {
    for height in 0..4 {
        let layout = compute(Rect::new(0, 0, 100, height));
        assert!(layout.app.height <= height);
        assert!(layout.stage.height <= height);
    }
}

#[test]
fn a_centred_box_too_big_for_the_frame_is_shrunk_to_fit() {
    let area = centered(200, 200, Rect::new(0, 0, 30, 10));
    assert_eq!((area.width, area.height), (30, 10));
}

#[test]
fn a_header_is_taken_off_the_top() {
    let [header, rest] = stack_header(Rect::new(0, 0, 20, 10), 2);
    assert_eq!((header.y, header.height), (0, 2));
    assert_eq!((rest.y, rest.height), (2, 8));
}

#[test]
fn a_footer_is_taken_off_the_bottom() {
    let [body, footer] = stack_footer(Rect::new(0, 0, 20, 10), 1);
    assert_eq!((body.y, body.height), (0, 9));
    assert_eq!((footer.y, footer.height), (9, 1));
}

#[test]
fn a_footer_taller_than_the_box_takes_all_of_it() {
    let [body, footer] = stack_footer(Rect::new(0, 0, 20, 2), 5);
    assert_eq!(body.height, 0);
    assert_eq!(footer.height, 2);
}
