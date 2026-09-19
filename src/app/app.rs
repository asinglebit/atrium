use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use portable_pty::CommandBuilder;
use ratatui::{DefaultTerminal, Frame, layout::Rect};

use crate::{
    app::{draw, input::keys},
    core::pty::PtySession,
};

/// atrium's own chords all live behind this one key, so every other keystroke
/// can go straight through to the agent untouched. F12 is bound by nothing else
/// in this stack -- not sway, ghostty, tmux, vim or readline.
const LEADER: KeyCode = KeyCode::F(12);

pub struct App {
    session: PtySession,
    leader_armed: bool,
    should_quit: bool,
}

impl App {
    pub fn new(cmd: CommandBuilder, rows: u16, cols: u16) -> io::Result<Self> {
        let session = PtySession::spawn(cmd, rows, cols)?;
        Ok(Self { session, leader_armed: false, should_quit: false })
    }

    /// Where the agent is drawn. The sidebar will carve into this later; for now
    /// the agent gets everything.
    fn stage_area(full: Rect) -> Rect {
        full
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            let stage = Self::stage_area(Rect::from(terminal.size()?));
            self.session.resize(stage.height, stage.width)?;

            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    // Release arrives on terminals with the kitty protocol; without
                    // this filter every keystroke is sent twice.
                    Event::Key(key) if key.kind != KeyEventKind::Release => self.on_key(key)?,
                    Event::Paste(text) => self.session.write(&keys::encode_paste(&text))?,
                    _ => {},
                }
            }

            if !self.session.is_alive() {
                self.should_quit = true;
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let stage = Self::stage_area(frame.area());
        draw::stage::draw(frame, stage, &self.session);
    }

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        if self.leader_armed {
            self.leader_armed = false;
            // Pressing the leader twice passes it through to the agent.
            if key.code == LEADER {
                return self.send(key);
            }
            if key.code == KeyCode::Char('q') {
                self.should_quit = true;
            }
            return Ok(());
        }

        if key.code == LEADER && key.modifiers == KeyModifiers::NONE {
            self.leader_armed = true;
            return Ok(());
        }

        self.send(key)
    }

    fn send(&mut self, key: KeyEvent) -> io::Result<()> {
        match keys::encode(key) {
            Some(bytes) => self.session.write(&bytes),
            None => Ok(()),
        }
    }
}
