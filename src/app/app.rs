use std::{io, time::Duration};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect};

use crate::{
    app::{draw, input::keys, state::layout},
    core::{
        agent::{Agent, AgentSpec},
        registry::Registry,
    },
};

/// atrium's own chords all live behind this one key, so every other keystroke
/// can go straight through to the agent untouched. F12 is bound by nothing else
/// in this stack -- not sway, ghostty, tmux, vim or readline.
const LEADER: KeyCode = KeyCode::F(12);

pub struct App {
    registry: Registry,
    spec: AgentSpec,
    stage: Rect,
    leader_armed: bool,
    should_quit: bool,
}

impl App {
    pub fn new(spec: AgentSpec, rows: u16, cols: u16) -> io::Result<Self> {
        let (_, stage) = layout::split(Rect::new(0, 0, cols, rows));
        let mut registry = Registry::new();
        registry.push(Agent::spawn(&spec, stage.height, stage.width)?);
        Ok(Self { registry, spec, stage, leader_armed: false, should_quit: false })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            let (_, stage) = layout::split(Rect::from(terminal.size()?));
            self.stage = stage;
            self.registry.resize_all(stage.height, stage.width)?;
            self.registry.refresh();

            terminal.draw(|frame| self.draw(frame))?;

            if event::poll(Duration::from_millis(16))? {
                match event::read()? {
                    // Release arrives on terminals with the kitty protocol; without
                    // this filter every keystroke is sent twice.
                    Event::Key(key) if key.kind != KeyEventKind::Release => self.on_key(key)?,
                    Event::Paste(text) => self.send_bytes(&keys::encode_paste(&text))?,
                    _ => {},
                }
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let (sidebar, stage) = layout::split(frame.area());
        if let Some(area) = sidebar {
            draw::sidebar::draw(frame, area, &self.registry);
        }
        if let Some(agent) = self.registry.focused() {
            draw::stage::draw(frame, stage, agent.session());
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        if self.leader_armed {
            self.leader_armed = false;
            return self.on_leader_chord(key);
        }

        if key.code == LEADER && key.modifiers == KeyModifiers::NONE {
            self.leader_armed = true;
            return Ok(());
        }

        self.send(key)
    }

    fn on_leader_chord(&mut self, key: KeyEvent) -> io::Result<()> {
        // Pressing the leader twice passes it through to the agent.
        if key.code == LEADER {
            return self.send(key);
        }

        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('n') => self.hold_another()?,
            KeyCode::Char('x') => self.dismiss(),
            KeyCode::Char('j') | KeyCode::Down => self.registry.focus_next(),
            KeyCode::Char('k') | KeyCode::Up => self.registry.focus_prev(),
            KeyCode::Char(c @ '1'..='9') => {
                let index = c as usize - '1' as usize;
                self.registry.focus_at(index);
            },
            _ => {},
        }
        Ok(())
    }

    fn hold_another(&mut self) -> io::Result<()> {
        let agent = Agent::spawn(&self.spec, self.stage.height, self.stage.width)?;
        self.registry.push(agent);
        Ok(())
    }

    /// Dropping the last agent ends atrium: it exists to hold them, so holding
    /// none leaves nothing to show.
    fn dismiss(&mut self) {
        self.registry.dismiss_focused();
        if self.registry.is_empty() {
            self.should_quit = true;
        }
    }

    fn send(&mut self, key: KeyEvent) -> io::Result<()> {
        match keys::encode(key) {
            Some(bytes) => self.send_bytes(&bytes),
            None => Ok(()),
        }
    }

    /// Input goes only to the focused agent, and only while it can still read it.
    fn send_bytes(&mut self, bytes: &[u8]) -> io::Result<()> {
        match self.registry.focused_mut() {
            Some(agent) if !agent.has_exited() => agent.write(bytes),
            _ => Ok(()),
        }
    }
}
