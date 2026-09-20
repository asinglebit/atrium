use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// The same frames guitar spins, so the two tools look related.
const FRAMES: [char; 6] = ['⠋', '⠙', '⠸', '⠴', '⠦', '⠇'];

/// How long each frame is held.
const FRAME_MS: u128 = 100;

/// Derived from elapsed time rather than driven by a thread, because the draw
/// loop already runs often enough to animate it.
pub fn frame_at(elapsed: Duration) -> char {
    FRAMES[(elapsed.as_millis() / FRAME_MS) as usize % FRAMES.len()]
}

/// How long the pulse holds each half of its cycle.
const PULSE_MS: u128 = 500;

/// Whether a pulsing thing is lit right now. Read from the wall clock rather
/// than from this process' own uptime, so two atriums -- and the tmuxbar window
/// they are both being drawn in -- light up together instead of each keeping
/// its own beat.
pub fn pulse_at(since_epoch: Duration) -> bool {
    (since_epoch.as_millis() / PULSE_MS).is_multiple_of(2)
}

/// The pulse now. Before the epoch is not a time anything runs at, so a clock
/// set that far back holds the lit half rather than failing.
pub fn pulse_now() -> bool {
    pulse_at(SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or(Duration::ZERO))
}

#[cfg(test)]
#[path = "../tests/helpers/spinner.rs"]
mod tests;
