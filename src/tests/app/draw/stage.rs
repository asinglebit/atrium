use super::*;
use crate::helpers::palette::THEME_PRESETS;
use portable_pty::CommandBuilder;
use ratatui::{Terminal, backend::TestBackend};

/// `cat` sits there with a blank screen, which is every cell defaulted -- the
/// case this is about.
fn session() -> PtySession {
    PtySession::spawn(CommandBuilder::new("cat"), 6, 20).expect("pty should open")
}

#[test]
fn the_agents_blank_screen_takes_the_themes_background() {
    let theme = Theme::classic();
    let session = session();
    let mut terminal = Terminal::new(TestBackend::new(20, 6)).expect("test terminal");

    terminal.draw(|frame| draw(frame, frame.area(), &session, &theme)).expect("draw");

    let buffer = terminal.backend().buffer();
    for x in 0..20 {
        for y in 0..6 {
            assert_eq!(buffer[(x, y)].bg, theme.background_color(), "cell ({x}, {y}) is still the terminal's own background");
        }
    }
}

#[test]
fn the_background_follows_whichever_theme_is_in_use() {
    let session = session();

    for preset in THEME_PRESETS.iter().take(4) {
        let mut terminal = Terminal::new(TestBackend::new(4, 2)).expect("test terminal");
        terminal.draw(|frame| draw(frame, frame.area(), &session, &preset.theme)).expect("draw");

        assert_eq!(terminal.backend().buffer()[(0, 0)].bg, preset.theme.background_color(), "{} did not reach the stage", preset.label);
    }
}
