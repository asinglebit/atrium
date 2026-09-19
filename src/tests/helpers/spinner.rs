use super::*;

#[test]
fn it_starts_on_the_first_frame() {
    assert_eq!(frame_at(Duration::ZERO), FRAMES[0]);
}

#[test]
fn it_advances_one_frame_per_interval() {
    assert_eq!(frame_at(Duration::from_millis(FRAME_MS as u64)), FRAMES[1]);
    assert_eq!(frame_at(Duration::from_millis(FRAME_MS as u64 * 2)), FRAMES[2]);
}

#[test]
fn it_wraps_instead_of_running_off_the_end() {
    let full_cycle = FRAME_MS as u64 * FRAMES.len() as u64;
    assert_eq!(frame_at(Duration::from_millis(full_cycle)), FRAMES[0]);
}

#[test]
fn a_long_uptime_still_yields_a_frame() {
    assert!(FRAMES.contains(&frame_at(Duration::from_secs(60 * 60 * 24))));
}
