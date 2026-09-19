use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame, layout::Rect, widgets::Block};

use crate::{
    app::{
        draw,
        input::{keymap::Keymap, keys},
        state::{goto::Goto, layout, picker::Picker},
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
    /// Some while a modal is up; input goes to it instead of the agent.
    modal: Option<Modal>,
    last_git: Instant,
    should_quit: bool,
}

/// Only one modal is ever up, and while it is, it owns the keyboard.
enum Modal {
    NewAgent(Picker),
    Goto(Goto),
}

/// What a key in the goto list amounts to, worked out before the registry is
/// touched so the modal is not still borrowed when the focus moves.
enum Jump {
    Close,
    To(usize),
    Stay,
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

        let stage = layout::compute(Rect::new(0, 0, cols, rows)).stage;
        let mut registry = Registry::new();
        registry.push(Agent::spawn(&spec, &harness, stage.height, stage.width)?);

        Ok(Self { registry, theme: config.theme, keymap: config.keymap, harness, server, started: Instant::now(), stage, modal: None, last_git: Instant::now(), should_quit: false })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            let stage = layout::compute(Rect::from(terminal.size()?)).stage;
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
        let layout = layout::compute(frame.area());
        let spinner = spinner::frame_at(self.started.elapsed());

        // The agent paints its own cells; this is what colours everything it
        // does not reach, so the chrome matches guitar rather than the terminal.
        frame.render_widget(Block::default().style(self.theme.background_style()), frame.area());
        frame.render_widget(draw::pane::app_frame(&self.theme), layout.app);

        let cwd = self.registry.focused().map_or_else(String::new, |agent| agent.cwd().display().to_string());
        draw::title::draw(frame, &layout, &self.theme, &cwd, self.view_name());

        if let Some(area) = layout.sidebar {
            draw::sidebar::draw(frame, area, &self.registry, &self.theme, spinner);
        }
        if let Some(agent) = self.registry.focused() {
            draw::stage::draw(frame, layout.stage, agent.session());
        }
        draw::statusbar::draw(frame, &layout, &self.registry, &self.theme);

        match &self.modal {
            Some(Modal::NewAgent(picker)) => draw::modals::new_agent::draw(frame, frame.area(), picker, &self.theme),
            Some(Modal::Goto(goto)) => draw::modals::goto::draw(frame, frame.area(), goto, &self.registry, &self.theme, spinner),
            None => {},
        }
    }

    /// What the top right corner says you are looking at.
    fn view_name(&self) -> &'static str {
        match self.modal {
            Some(Modal::NewAgent(_)) => "new agent",
            Some(Modal::Goto(_)) => "go to",
            None => "agents",
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        match self.modal {
            Some(Modal::NewAgent(_)) => return self.on_picker_key(key),
            Some(Modal::Goto(_)) => {
                self.on_goto_key(key);
                return Ok(());
            },
            None => {},
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
        } else if keymap.goto.matches(key) {
            self.modal = Some(Modal::Goto(Goto::new(self.registry.len(), self.registry.focus())));
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
        self.modal = Some(Modal::NewAgent(Picker::new(projects::discover(&projects::default_root()))));
    }

    /// The goto list is short and numbered, so a digit is the fast path and
    /// j/k is there for when the row has scrolled past nine.
    fn on_goto_key(&mut self, key: KeyEvent) {
        let jump = {
            let Some(Modal::Goto(goto)) = &mut self.modal else {
                return;
            };
            match key.code {
                KeyCode::Esc => Jump::Close,
                KeyCode::Enter => Jump::To(goto.selected()),
                KeyCode::Char(c) if c.is_ascii_digit() => goto.row_for(c).map_or(Jump::Stay, Jump::To),
                KeyCode::Char('j') | KeyCode::Down => {
                    goto.move_down();
                    Jump::Stay
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    goto.move_up();
                    Jump::Stay
                },
                _ => Jump::Stay,
            }
        };

        match jump {
            Jump::Close => self.modal = None,
            Jump::To(index) => {
                self.registry.focus_at(index);
                self.modal = None;
            },
            Jump::Stay => {},
        }
    }

    /// While the modal is up it owns every key: nothing reaches the agent, so a
    /// stray keystroke cannot land in a conversation you cannot see.
    fn on_picker_key(&mut self, key: KeyEvent) -> io::Result<()> {
        let Some(Modal::NewAgent(picker)) = &mut self.modal else {
            return Ok(());
        };
        match key.code {
            KeyCode::Esc => self.modal = None,
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
        let Some(Modal::NewAgent(picker)) = &self.modal else {
            return Ok(());
        };
        let Some(project) = picker.selected_project() else {
            return Ok(());
        };

        let spec = AgentSpec::new(picker.kind(), Vec::new(), project.path.clone());
        match Agent::spawn(&spec, &self.harness, self.stage.height, self.stage.width) {
            Ok(agent) => {
                self.registry.push(agent);
                self.modal = None;
            },
            Err(error) => {
                if let Some(Modal::NewAgent(picker)) = &mut self.modal {
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
