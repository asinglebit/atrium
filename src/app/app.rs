use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{DefaultTerminal, Frame, layout::Rect, widgets::Block};

use crate::{
    app::{
        draw,
        input::{keymap::Keymap, keys},
        state::{
            goto::Goto,
            layout,
            layout::Layout,
            menu::{Action, Menu},
            picker::Picker,
            settings::Settings,
        },
    },
    core::{
        agent::{Agent, AgentSpec, Harness},
        config::Config,
        layout_config::{self, LayoutConfig},
        projects,
        registry::Registry,
    },
    helpers::{palette, palette::Theme, scroll, spinner},
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
    /// Some while the settings view has replaced the panes.
    settings: Option<Settings>,
    /// Some while a right-click menu is up; it owns the keyboard and the mouse.
    menu: Option<Menu>,
    sidebar_visible: bool,
    sidebar_scroll: usize,
    /// How wide the sidebar is asked to be. Remembered across runs.
    sidebar_width: u16,
    /// True from grabbing the line between the panes until the button is let go.
    dragging_sidebar: bool,
    /// The last layout drawn, which is what a click is measured against.
    layout: Layout,
    last_git: Instant,
    should_quit: bool,
}

/// Only one modal is ever up, and while it is, it owns the keyboard.
enum Modal {
    NewAgent(Picker),
    Goto(Goto),
}

/// What an input did to the menu, worked out while the menu is borrowed so that
/// acting on it can happen once the borrow is gone.
enum Picked {
    Nothing,
    Close,
    Act(Action),
}

/// What a key in the goto list amounts to, worked out before the registry is
/// touched so the modal is not still borrowed when the focus moves.
enum Jump {
    Close,
    To(usize),
    Stay,
}

/// Whether a point falls inside a rectangle.
fn within(area: Rect, (column, row): (u16, u16)) -> bool {
    column >= area.x && column < area.x + area.width && row >= area.y && row < area.y + area.height
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

        let sidebar_width = layout_config::load().sidebar_width;
        let stage = layout::compute(Rect::new(0, 0, cols, rows), true, sidebar_width).stage;
        let mut registry = Registry::new();
        registry.push(Agent::spawn(&spec, &harness, stage.height, stage.width)?);

        Ok(Self {
            registry,
            theme: config.theme,
            keymap: config.keymap,
            harness,
            server,
            started: Instant::now(),
            stage,
            modal: None,
            settings: None,
            menu: None,
            sidebar_visible: true,
            sidebar_scroll: 0,
            sidebar_width,
            dragging_sidebar: false,
            layout: layout::compute(Rect::new(0, 0, cols, rows), true, sidebar_width),
            last_git: Instant::now(),
            should_quit: false,
        })
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.should_quit {
            let stage = self.layout_for(Rect::from(terminal.size()?)).stage;
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
                    Event::Mouse(mouse) => self.on_mouse(mouse)?,
                    Event::Paste(text) => self.send_bytes(&keys::encode_paste(&text))?,
                    _ => {},
                }
            }
        }
        Ok(())
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let layout = self.layout_for(frame.area());
        self.layout = layout;
        let spinner = spinner::frame_at(self.started.elapsed());

        // The agent paints its own cells; this is what colours everything it
        // does not reach, so the chrome matches guitar rather than the terminal.
        frame.render_widget(Block::default().style(self.theme.background_style()), frame.area());
        frame.render_widget(draw::pane::app_frame(&self.theme), layout.app);

        let cwd = self.registry.focused().map_or_else(String::new, |agent| agent.cwd().display().to_string());
        draw::title::draw(frame, &layout, &self.theme, &cwd, self.view_name());

        if let Some(settings) = &mut self.settings {
            draw::settings::draw(frame, layout.stage, settings, &self.keymap, &self.theme);
        } else {
            if let Some(area) = layout.sidebar {
                self.sidebar_scroll = scroll::trap(self.registry.focus(), self.sidebar_scroll, self.registry.len(), area.height as usize);
                draw::sidebar::draw(frame, area, &self.registry, &self.theme, spinner, self.sidebar_scroll);
            }
            if let Some(agent) = self.registry.focused() {
                draw::stage::draw(frame, layout.stage, agent.session());
            }
        }
        draw::statusbar::draw(frame, &layout, &self.registry, &self.theme);

        match &self.modal {
            Some(Modal::NewAgent(picker)) => draw::modals::new_agent::draw(frame, frame.area(), picker, &self.theme),
            Some(Modal::Goto(goto)) => draw::modals::goto::draw(frame, frame.area(), goto, &self.registry, &self.theme, spinner),
            None => {},
        }

        // Last, so it floats over whatever was right-clicked.
        if let Some(menu) = &self.menu {
            draw::menu::draw(frame, frame.area(), menu, &self.theme);
        }
    }

    /// The settings view takes the whole inside, so it hides the sidebar for as
    /// long as it is open without changing what the sidebar key last said.
    fn layout_for(&self, full: Rect) -> Layout {
        layout::compute(full, self.sidebar_visible && self.settings.is_none(), self.sidebar_width)
    }

    /// What the top right corner says you are looking at.
    fn view_name(&self) -> &'static str {
        match self.modal {
            Some(Modal::NewAgent(_)) => "new agent",
            Some(Modal::Goto(_)) => "go to",
            None if self.settings.is_some() => "settings",
            None => "agents",
        }
    }

    fn on_key(&mut self, key: KeyEvent) -> io::Result<()> {
        if self.menu.is_some() {
            self.on_menu_key(key);
            return Ok(());
        }
        if self.settings.is_some() {
            self.on_settings_key(key);
            return Ok(());
        }
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
        } else if keymap.settings.matches(key) {
            self.settings = Some(Settings::new(&self.theme));
        } else if keymap.sidebar.matches(key) {
            self.sidebar_visible = !self.sidebar_visible;
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

    /// Clicks and the wheel go wherever they landed. Anything inside the stage
    /// is re-encoded and handed to the agent, which asked the terminal for the
    /// mouse itself and has no idea there is a sidebar beside it.
    fn on_mouse(&mut self, mouse: MouseEvent) -> io::Result<()> {
        let layout = self.layout;
        let at = (mouse.column, mouse.row);

        // A drag that began on the sidebar's edge keeps the mouse until the
        // button is let go, however far outside the sidebar it has wandered.
        if self.dragging_sidebar {
            match mouse.kind {
                MouseEventKind::Drag(MouseButton::Left) => self.resize_sidebar(mouse.column),
                MouseEventKind::Up(MouseButton::Left) => {
                    self.dragging_sidebar = false;
                    // Written once the drag settles rather than on every frame
                    // of it, so one resize is one write.
                    layout_config::save(&LayoutConfig { sidebar_width: self.sidebar_width });
                },
                _ => {},
            }
            return Ok(());
        }
        // The menu owns the mouse for as long as it is up.
        if self.menu.is_some() {
            return self.on_menu_mouse(mouse);
        }
        if mouse.kind == MouseEventKind::Down(MouseButton::Right) {
            self.open_menu(at);
            return Ok(());
        }
        if self.settings.is_some() {
            self.on_settings_mouse(mouse, layout.stage);
            return Ok(());
        }
        if self.modal.is_some() {
            self.on_modal_mouse(mouse);
            return Ok(());
        }
        if let Some(sidebar) = layout.sidebar
            && within(sidebar, at)
        {
            self.on_sidebar_mouse(mouse, sidebar);
            return Ok(());
        }
        if within(layout.stage, at)
            && let Some(bytes) = keys::encode_mouse(&mouse, (layout.stage.x, layout.stage.y))
        {
            return self.send_bytes(&bytes);
        }
        Ok(())
    }

    fn on_sidebar_mouse(&mut self, mouse: MouseEvent, area: Rect) {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.sidebar_scroll = self.sidebar_scroll.saturating_add(1),
            MouseEventKind::ScrollUp => self.sidebar_scroll = self.sidebar_scroll.saturating_sub(1),
            MouseEventKind::Down(MouseButton::Left) => {
                // The sidebar's last column is the line between the panes, so
                // that is what there is to grab.
                if mouse.column == area.x.saturating_add(area.width).saturating_sub(1) {
                    self.dragging_sidebar = true;
                } else if let Some(index) = draw::sidebar::row_at(area, self.sidebar_scroll, mouse.row) {
                    self.registry.focus_at(index);
                }
            },
            _ => {},
        }
    }

    /// The column under the cursor becomes the sidebar's last column, which is
    /// the line being dragged.
    fn resize_sidebar(&mut self, column: u16) {
        let left = self.layout.app.x.saturating_add(1);
        let want = column.saturating_sub(left).saturating_add(1);
        self.sidebar_width = layout::clamp_sidebar(want, self.layout.app.width.saturating_sub(2));
    }

    fn on_settings_mouse(&mut self, mouse: MouseEvent, area: Rect) {
        let Some(settings) = &mut self.settings else {
            return;
        };
        match mouse.kind {
            MouseEventKind::ScrollDown => settings.move_down(),
            MouseEventKind::ScrollUp => settings.move_up(),
            MouseEventKind::Down(MouseButton::Left) => {
                if let Some(tab) = draw::settings::tab_at(settings, area, mouse.column, mouse.row) {
                    settings.open(tab);
                } else if let Some(index) = draw::settings::row_at(settings, area, mouse.row) {
                    settings.select(index);
                }
            },
            _ => {},
        }
    }

    /// Modals are small and centred; the wheel moves the cursor rather than
    /// asking each one to hit-test its own geometry.
    fn on_modal_mouse(&mut self, mouse: MouseEvent) {
        let down = match mouse.kind {
            MouseEventKind::ScrollDown => true,
            MouseEventKind::ScrollUp => false,
            _ => return,
        };
        match &mut self.modal {
            Some(Modal::NewAgent(picker)) if down => picker.move_down(),
            Some(Modal::NewAgent(picker)) => picker.move_up(),
            Some(Modal::Goto(goto)) if down => goto.move_down(),
            Some(Modal::Goto(goto)) => goto.move_up(),
            None => {},
        }
    }

    /// The settings view owns the keyboard while it is open, the same way a
    /// modal does -- nothing reaches the agent behind it.
    fn on_settings_key(&mut self, key: KeyEvent) {
        let Some(settings) = &mut self.settings else {
            return;
        };
        match key.code {
            KeyCode::Esc => self.settings = None,
            KeyCode::Tab | KeyCode::Right => settings.next_tab(),
            KeyCode::BackTab | KeyCode::Left => settings.previous_tab(),
            KeyCode::Char('j') | KeyCode::Down => settings.move_down(),
            KeyCode::Char('k') | KeyCode::Up => settings.move_up(),
            KeyCode::Enter => {
                if let Some(theme) = settings.theme_under_cursor() {
                    self.theme = theme;
                    // Written to atrium's own theme.json, never guitar's.
                    palette::save_theme(&theme);
                }
            },
            _ => {},
        }
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
        self.dismiss_at(self.registry.focus());
    }

    fn dismiss_at(&mut self, index: usize) {
        self.registry.dismiss_at(index);
        if self.registry.is_empty() {
            self.should_quit = true;
        }
    }

    /// A right-click on a row offers what can be done to that agent; anywhere
    /// else offers what can be done regardless of where you clicked.
    fn open_menu(&mut self, at: (u16, u16)) {
        if self.settings.is_some() || self.modal.is_some() {
            return;
        }

        let clicked = self
            .layout
            .sidebar
            .filter(|sidebar| within(*sidebar, at))
            .and_then(|sidebar| draw::sidebar::row_at(sidebar, self.sidebar_scroll, at.1))
            .and_then(|index| self.registry.agents().get(index).map(|agent| (index, agent.name.clone())));

        self.menu = Some(match clicked {
            Some((index, name)) => Menu::for_agent(at, index, &name, &self.keymap),
            None => {
                let focused = self.registry.focused().map(|agent| (self.registry.focus(), agent.name.clone()));
                Menu::general(at, focused.as_ref().map(|(index, name)| (*index, name.as_str())), &self.keymap)
            },
        });
    }

    /// While the menu is up it owns the keyboard, the same way a modal does.
    fn on_menu_key(&mut self, key: KeyEvent) {
        let picked = {
            let Some(menu) = &mut self.menu else {
                return;
            };
            match key.code {
                KeyCode::Esc => Picked::Close,
                KeyCode::Enter => menu.picked().map_or(Picked::Close, Picked::Act),
                KeyCode::Char('j') | KeyCode::Down => {
                    menu.move_down();
                    Picked::Nothing
                },
                KeyCode::Char('k') | KeyCode::Up => {
                    menu.move_up();
                    Picked::Nothing
                },
                _ => Picked::Nothing,
            }
        };
        self.settle(picked);
    }

    fn on_menu_mouse(&mut self, mouse: MouseEvent) -> io::Result<()> {
        let full = self.layout.full;
        let (column, row) = (mouse.column, mouse.row);

        let picked = {
            let Some(menu) = &mut self.menu else {
                return Ok(());
            };
            match mouse.kind {
                MouseEventKind::ScrollDown => {
                    menu.move_down();
                    Picked::Nothing
                },
                MouseEventKind::ScrollUp => {
                    menu.move_up();
                    Picked::Nothing
                },
                // The cursor moving over an entry selects it, so what a click
                // will do is always the thing under the pointer.
                MouseEventKind::Moved => {
                    if let Some(index) = menu.item_at(full, column, row) {
                        menu.select(index);
                    }
                    Picked::Nothing
                },
                MouseEventKind::Down(_) => match menu.item_at(full, column, row) {
                    Some(index) => {
                        menu.select(index);
                        menu.picked().map_or(Picked::Close, Picked::Act)
                    },
                    // A click on the border is still a click on the menu.
                    None if menu.covers(full, column, row) => Picked::Nothing,
                    None => Picked::Close,
                },
                _ => Picked::Nothing,
            }
        };
        self.settle(picked);
        Ok(())
    }

    /// Closes the menu if the last input finished with it, and does whatever it
    /// was asked for once the borrow on it is gone.
    fn settle(&mut self, picked: Picked) {
        match picked {
            Picked::Nothing => {},
            Picked::Close => self.menu = None,
            Picked::Act(action) => {
                self.menu = None;
                self.take_menu_action(action);
            },
        }
    }

    fn take_menu_action(&mut self, action: Action) {
        match action {
            Action::Focus(index) => self.registry.focus_at(index),
            Action::Dismiss(index) => self.dismiss_at(index),
            Action::New => self.open_picker(),
            Action::Sidebar => self.sidebar_visible = !self.sidebar_visible,
            Action::Settings => self.settings = Some(Settings::new(&self.theme)),
            Action::Quit => self.should_quit = true,
            Action::Separator => {},
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
