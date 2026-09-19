use std::time::Duration;

/// The same frames guitar spins, so the two tools look related.
const FRAMES: [char; 6] = ['⠋', '⠙', '⠸', '⠴', '⠦', '⠇'];

/// How long each frame is held.
const FRAME_MS: u128 = 100;

/// Derived from elapsed time rather than driven by a thread, because the draw
/// loop already runs often enough to animate it.
pub fn frame_at(elapsed: Duration) -> char {
    FRAMES[(elapsed.as_millis() / FRAME_MS) as usize % FRAMES.len()]
}

#[cfg(test)]
#[path = "../tests/helpers/spinner.rs"]
mod tests;
