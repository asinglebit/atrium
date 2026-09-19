use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect};

use crate::{
    app::{
        draw,
        input::{keymap::Keymap, keys},
        state::{layout, picker::Picker},
    },
    core::{
        agent::{Agent, AgentSpec, Harness},
        config::Config,
        projects,
        registry::Registry,
    },
    helpers::{palette::Theme, spinner},
    ipc::server::StatusServer,
};

/// How often the branch and dirty flag are re-read. Slow enough that git status
/// on a big repo never shows up as a stutter.
const GIT_INTERVAL: Duration = Duration::from_secs(3);

pub struct App {
    registry: Registry,
    theme: Theme,
    keymap: Keymap,
    harness: Harness,
    server: StatusServer,
    started: Instant,
    stage: Rect,
    /// Some while the new-agent modal is up; input goes to it instead of the agent.
    picker: Option<Picker>,
    last_git: Instant,
    should_quit: bool,
}

/// Spawn failures arrive as a paragraph naming every directory on PATH; only
/// the first line says anything the reader needs.
fn first_line(message: &str) -> String {
    message.lines().next().unwrap_or(message).to_owned()
}

impl App {
    pub fn new(spec: AgentSpec, config: Config, rows: u16, cols: u16) -> io::Result<Self> {
        let server = StatusServer::bind()?;
        // Agents call back into this same binary, so its path is the one to hand out.
        let harness = Harness { exe: std::env::current_exe()?, socket: server.path().to_path_buf() };

        let (_, stage) = layout::split(Rect::new(0, 0, cols, rows));
        let mut registry = Registry::new();
        registry.push(Agent::spawn(&spec, &harness, stage.height, stage.width)?);

        Ok(Self { registry, theme: config.theme, keymap: config.keymap, harness, server, started: Instant::now(), stage, picker: None, last_git: Instant::now(), should_quit: false })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            let (_, stage) = layout::split(Rect::from(terminal.size()?));
            self.stage = stage;
            self.registry.resize_all(stage.height, stage.width)?;
            for report in self.server.drain() {
                self.registry.apply(&report);
            }
            self.registry.refresh();
            if self.last_git.elapsed() >= GIT_INTERVAL {
                self.registry.refresh_git();
                self.last_git = Instant::now();
            }

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
        // The agent paints its own cells; this is what colours everything it
        // does not reach, so the chrome matches guitar rather than the terminal.
        frame.render_widget(ratatui::widgets::Block::default().style(self.theme.background_style()), frame.area());

        let (sidebar, stage) = layout::split(frame.area());
        if let Some(area) = sidebar {
            draw::sidebar::draw(frame, area, &self.registry, &self.theme, spinner::frame_at(self.started.elapsed()));
        }
        if let Some(agent) = self.registry.focused() {
            draw::stage::draw(frame, stage, agent.session());
        }
        if let Some(picker) = &self.picker {
            draw::modals::new_agent::draw(frame, frame.area(), picker, &self.theme);
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        if self.picker.is_some() {
            return self.on_picker_key(key);
        }
        if self.take_action(&key) {
            return Ok(());
        }
        self.send(key)
    }

    /// True when atrium kept the key for itself. Anything it does not claim
    /// goes straight through, which is most of the keyboard.
    fn take_action(&mut self, key: &KeyEvent) -> bool {
        let keymap = self.keymap;
        if keymap.quit.matches(key) {
            self.should_quit = true;
        } else if keymap.new.matches(key) {
            self.open_picker();
        } else if keymap.dismiss.matches(key) {
            self.dismiss();
        } else if keymap.next.matches(key) {
            self.registry.focus_next();
        } else if keymap.previous.matches(key) {
            self.registry.focus_prev();
        } else {
            return false;
        }
        true
    }

    fn open_picker(&mut self) {
        self.picker = Some(Picker::new(projects::discover(&projects::default_root())));
    }

    /// While the modal is up it owns every key: nothing reaches the agent, so a
    /// stray keystroke cannot land in a conversation you cannot see.
    fn on_picker_key(&mut self, key: KeyEvent) -> io::Result<()> {
        let Some(picker) = &mut self.picker else {
            return Ok(());
        };
        match key.code {
            KeyCode::Esc => self.picker = None,
            KeyCode::Enter => return self.hold_picked(),
            KeyCode::Tab => picker.cycle_kind(),
            KeyCode::Down => picker.move_down(),
            KeyCode::Up => picker.move_up(),
            KeyCode::Backspace => picker.backspace(),
            KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => picker.push(c),
            _ => {},
        }
        Ok(())
    }

    /// A CLI that is not installed is a message, not the end of atrium, so the
    /// failure stays inside the modal instead of propagating out of the loop.
    fn hold_picked(&mut self) -> io::Result<()> {
        let Some(picker) = &self.picker else {
            return Ok(());
        };
        let Some(project) = picker.selected_project() else {
            return Ok(());
        };

        let spec = AgentSpec::new(picker.kind(), Vec::new(), project.path.clone());
        match Agent::spawn(&spec, &self.harness, self.stage.height, self.stage.width) {
            Ok(agent) => {
                self.registry.push(agent);
                self.picker = None;
            },
            Err(error) => {
                if let Some(picker) = &mut self.picker {
                    picker.set_error(first_line(&error.to_string()));
                }
            },
        }
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
