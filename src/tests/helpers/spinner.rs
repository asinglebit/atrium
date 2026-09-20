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

#[test]
fn the_pulse_starts_lit() {
    assert!(pulse_at(Duration::ZERO));
}

#[test]
fn the_pulse_turns_over_every_half_of_its_cycle() {
    assert!(!pulse_at(Duration::from_millis(PULSE_MS as u64)));
    assert!(pulse_at(Duration::from_millis(PULSE_MS as u64 * 2)));
}

#[test]
fn two_instants_in_one_half_agree() {
    let half = PULSE_MS as u64;
    assert_eq!(pulse_at(Duration::from_millis(half + 1)), pulse_at(Duration::from_millis(half * 2 - 1)));
}

#[test]
fn the_wall_clock_is_what_two_atriums_share() {
    // Same instant, two callers: the answer cannot depend on who is asking.
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();
    assert_eq!(pulse_at(now), pulse_at(now));
}
