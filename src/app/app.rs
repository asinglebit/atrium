use std::{
    io,
    time::{Duration, Instant},
};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::{DefaultTerminal, Frame, layout::Rect, widgets::Block};

use crate::{
    adapters,
    app::{
        draw,
        input::{keymap::Keymap, keys},
        state::{
            goto::Goto,
            layout,
            layout::Layout,
            menu::{Action, Menu},
            picker::Picker,
            profile_editor::{Editor, Outcome},
            settings::{SelectionKind, Settings},
            splash::Splash,
        },
    },
    core::{
        agent::{Agent, AgentSpec, Harness},
        config::Config,
        installed::Found,
        layout_config::{self, LayoutConfig},
        profile::Profile,
        profiles_file::{self, StoredProfiles},
        projects,
        registry::Registry,
    },
    helpers::{
        palette::{self, THEME_PRESETS, Theme},
        scroll, spinner,
    },
    ipc::server::StatusServer,
};

/// How often the branch and dirty flag are re-read. Slow enough that git status
/// on a big repo never shows up as a stutter.
const GIT_INTERVAL: Duration = Duration::from_secs(3);

pub struct App {
    registry: Registry,
    theme: Theme,
    keymap: Keymap,
    /// What can be held, and which of them the picker opens on.
    profiles: Vec<Profile>,
    default_profile: usize,
    /// The profiles as written down, which is what the settings view edits.
    stored_profiles: StoredProfiles,
    /// Which of the CLIs atrium knows this machine has, scanned once at start.
    /// Kept so an edited set can be merged against it without a rescan.
    installed: Vec<Found>,
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
    /// Some while a profile is being added or changed, over the settings view.
    profile_editor: Option<Editor>,
    /// What is shown while nothing is held: the wordmark and what could be.
    splash: Splash,
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
    /// `spec` is what to hold straight away. None opens on the splash instead,
    /// which is what a bare `atrium` does -- nothing was asked for, so it asks.
    pub fn new(spec: Option<AgentSpec>, config: Config, rows: u16, cols: u16) -> io::Result<Self> {
        let server = StatusServer::bind()?;
        // Agents call back into this same binary, so its path is the one to hand out.
        let harness = Harness { exe: std::env::current_exe()?, socket: server.path().to_path_buf() };

        let sidebar_width = layout_config::load().sidebar_width;
        let stage = layout::compute(Rect::new(0, 0, cols, rows), true, sidebar_width).stage;
        let mut registry = Registry::new();
        if let Some(spec) = spec {
            registry.push(Agent::spawn(&spec, &harness, &config.theme, stage.height, stage.width)?);
        }
        let splash = Splash::new(config.profiles.len(), config.default_profile);

        Ok(Self {
            registry,
            theme: config.theme,
            keymap: config.keymap,
            profiles: config.profiles,
            default_profile: config.default_profile,
            stored_profiles: config.stored_profiles,
            installed: config.installed,
            harness,
            server,
            started: Instant::now(),
            stage,
            modal: None,
            settings: None,
            menu: None,
            profile_editor: None,
            splash,
            // Hidden until asked for: one agent is the common case, and a
            // sidebar listing it alone says nothing the status line does not.
            sidebar_visible: false,
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

        // The splash takes the bare terminal: no frame, no title line, no status
        // line. Nothing is held, so none of them would have anything to say.
        if self.is_bare() {
            draw::splash::draw(frame, frame.area(), &self.splash, &self.profiles, &self.theme);
        } else {
            frame.render_widget(draw::pane::app_frame(&self.theme), layout.app);

            let cwd = self.registry.focused().map_or_else(String::new, |agent| agent.cwd().display().to_string());
            draw::title::draw(frame, &layout, &self.theme, &cwd, self.view_name());

            if let Some(settings) = &mut self.settings {
                let context = draw::settings::Context {
                    keymap: &self.keymap,
                    theme: &self.theme,
                    profiles: &self.stored_profiles,
                    installed: &self.installed,
                    default_profile: self.default_profile,
                    socket: self.server.path(),
                };
                draw::settings::draw(frame, layout.stage, layout.app, settings, &context);
            } else {
                if let Some(area) = layout.sidebar {
                    self.sidebar_scroll = scroll::trap(self.registry.focus(), self.sidebar_scroll, self.registry.len(), area.height as usize);
                    draw::sidebar::draw(frame, area, &self.registry, &self.theme, spinner, self.sidebar_scroll);
                }
                if let Some(agent) = self.registry.focused() {
                    draw::stage::draw(frame, layout.stage, agent.session(), &self.theme);
                }
            }
            draw::statusbar::draw(frame, &layout, &self.registry, &self.theme);
        }

        match &self.modal {
            Some(Modal::NewAgent(picker)) => draw::modals::new_agent::draw(frame, frame.area(), picker, &self.theme),
            Some(Modal::Goto(goto)) => draw::modals::goto::draw(frame, frame.area(), goto, &self.registry, &self.theme, spinner),
            None => {},
        }

        if let Some(editor) = &self.profile_editor {
            let name = Editor::name_of(&self.stored_profiles, self.editing_index());
            draw::modals::profile::draw(frame, frame.area(), editor, &name, &self.theme);
        }

        // Last, so it floats over whatever was right-clicked.
        if let Some(menu) = &self.menu {
            draw::menu::draw(frame, frame.area(), menu, &self.theme);
        }
    }

    /// True while the splash is what is on screen, which is the one view that
    /// wears no chrome at all. Settings over an empty registry is not bare: it
    /// is a view of something, and keeps the frame around it.
    fn is_bare(&self) -> bool {
        self.registry.is_empty() && self.settings.is_none()
    }

    /// The settings view takes the whole inside, so it hides the sidebar for as
    /// long as it is open without changing what the sidebar key last said.
    fn layout_for(&self, full: Rect) -> Layout {
        // The splash takes the whole inside, the way the settings view does --
        // there are no rows to put beside it.
        let sidebar = self.sidebar_visible && self.settings.is_none() && !self.registry.is_empty();
        layout::compute(full, sidebar, self.sidebar_width)
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
        if self.profile_editor.is_some() {
            self.on_editor_key(key);
            return Ok(());
        }
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
        // With nothing held there is no agent for a key to reach, so the splash
        // takes what the chords above did not.
        if self.registry.is_empty() {
            return self.on_splash_key(key);
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
            self.settings = Some(Settings::new());
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
        if self.registry.is_empty() {
            self.on_splash_mouse(mouse, layout.full);
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
                // Measured against the line the last draw put there, so a click
                // lands on what was painted rather than on recomputed geometry.
                let line = settings.scroll + usize::from(mouse.row.saturating_sub(area.y));
                match settings.tab_at(line, mouse.column) {
                    Some(tab) => settings.open(tab),
                    None => settings.select_line(line),
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
        // Closing is settled before the borrow below, so the rest of this can
        // work on the state without having to hand it back first.
        if key.code == KeyCode::Esc {
            self.settings = None;
            return;
        }

        let chosen = {
            let Some(settings) = &mut self.settings else {
                return;
            };
            match key.code {
                KeyCode::Tab | KeyCode::Right => settings.next_tab(),
                KeyCode::BackTab | KeyCode::Left => settings.previous_tab(),
                KeyCode::Char('j') | KeyCode::Down => settings.move_down(),
                KeyCode::Char('k') | KeyCode::Up => settings.move_up(),
                KeyCode::Enter => return self.settle_cursor(),
                _ => {},
            }
            None
        };
        self.settle_setting(chosen);
    }

    /// Enter, worked out and then acted on in two steps so the settings state
    /// is not still borrowed when the theme changes under it.
    fn settle_cursor(&mut self) {
        let chosen = self.settings.as_ref().and_then(|settings| settings.kind_at_cursor().cloned());
        self.settle_setting(chosen);
    }

    /// Acts on whatever the cursor was sitting on, once the borrow on the
    /// settings state is gone.
    fn settle_setting(&mut self, kind: Option<SelectionKind>) {
        match kind {
            Some(SelectionKind::Theme(index)) => {
                if let Some(preset) = THEME_PRESETS.get(index) {
                    self.theme = preset.theme;
                    // Written to atrium's own theme.json, never guitar's.
                    palette::save_theme(&preset.theme);
                    self.retheme_agents();
                }
            },
            Some(SelectionKind::AddProfile) => self.profile_editor = Some(Editor::add()),
            Some(SelectionKind::Profile(index)) => self.profile_editor = Some(Editor::manage(index)),
            // Info rows and chords can be landed on but do nothing: paths are
            // there to be read, and a chord is rebound in the config file.
            _ => {},
        }
    }

    /// Hands the theme just chosen to everything already held. A claude reads
    /// its theme file again and repaints; an opencode keeps what it started
    /// with, so for that one this is the next agent's colours being got ready.
    fn retheme_agents(&self) {
        for agent in self.registry.agents() {
            adapters::detect(&agent.program).retheme(agent.config_dir.as_deref(), &self.theme);
        }
    }

    /// Which profile the editor is working on, for the name in its title.
    fn editing_index(&self) -> usize {
        match &self.profile_editor {
            Some(editor) => editor.index().unwrap_or(0),
            None => 0,
        }
    }

    /// While the editor is up it owns the keyboard, the way every modal here
    /// does -- nothing reaches the settings view behind it, let alone an agent.
    fn on_editor_key(&mut self, key: KeyEvent) {
        let outcome = {
            let Some(editor) = &mut self.profile_editor else {
                return;
            };
            match key.code {
                KeyCode::Esc => editor.cancel(),
                KeyCode::Enter => editor.confirm(&self.stored_profiles),
                KeyCode::Down => {
                    editor.move_down();
                    Outcome::Continue
                },
                KeyCode::Up => {
                    editor.move_up();
                    Outcome::Continue
                },
                KeyCode::Backspace => {
                    editor.backspace();
                    Outcome::Continue
                },
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    editor.push(c);
                    Outcome::Continue
                },
                _ => Outcome::Continue,
            }
        };
        self.settle_editor(outcome);
    }

    /// Applies what the editor asked for, writes it, and takes up the result.
    /// A refusal stays in the modal rather than closing it, so the answer can
    /// be corrected without starting again.
    fn settle_editor(&mut self, outcome: Outcome) {
        let Outcome::Commit(apply) = outcome else {
            if matches!(outcome, Outcome::Close) {
                self.profile_editor = None;
            }
            return;
        };

        let mut profiles = self.stored_profiles.clone();
        match apply(&mut profiles).and_then(|()| profiles_file::save(&profiles)) {
            Ok(()) => {
                self.adopt_profiles(profiles);
                self.profile_editor = None;
            },
            Err(message) => {
                if let Some(editor) = &mut self.profile_editor {
                    editor.fail(message);
                }
            },
        }
    }

    /// The splash offers whatever is configured, so an edited set changes it.
    fn refresh_splash(&mut self) {
        if self.registry.is_empty() {
            self.splash = Splash::new(self.profiles.len(), self.default_profile);
        }
    }

    /// Takes up an edited set: the picker and every later spawn see it at once,
    /// without atrium having to be restarted.
    fn adopt_profiles(&mut self, stored: StoredProfiles) {
        let (profiles, default_profile) = stored.resolve(&self.installed);
        self.profiles = profiles;
        self.default_profile = default_profile;
        self.stored_profiles = stored;
        self.refresh_splash();
    }

    fn open_picker(&mut self) {
        self.modal = Some(Modal::NewAgent(Picker::new(projects::discover(&projects::default_root()), self.profiles.clone(), self.default_profile)));
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
            KeyCode::Tab => picker.cycle_profile(),
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
        let Some(profile) = picker.profile() else {
            return Ok(());
        };

        let spec = AgentSpec::from_profile(profile, project.path.clone());
        match Agent::spawn(&spec, &self.harness, &self.theme, self.stage.height, self.stage.width) {
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

    /// Closing the last agent falls back to the splash rather than ending
    /// atrium; `quit` is how you leave.
    fn dismiss_at(&mut self, index: usize) {
        self.registry.dismiss_at(index);
        if self.registry.is_empty() {
            self.splash = Splash::new(self.profiles.len(), self.default_profile);
        }
    }

    fn on_splash_mouse(&mut self, mouse: MouseEvent, area: Rect) {
        match mouse.kind {
            MouseEventKind::ScrollDown => self.splash.move_down(),
            MouseEventKind::ScrollUp => self.splash.move_up(),
            MouseEventKind::Down(MouseButton::Left) => {
                // A click on a harness holds it: the list exists to be picked
                // from, so selecting and then confirming would be two steps for
                // one intention.
                let first = draw::splash::first_row(area, &self.splash, self.profiles.len());
                if let Some(index) = self.splash.row_at(first, mouse.row) {
                    self.hold_from_splash(index);
                }
            },
            _ => {},
        }
    }

    fn on_splash_key(&mut self, key: KeyEvent) -> io::Result<()> {
        match key.code {
            KeyCode::Char('j') | KeyCode::Down => self.splash.move_down(),
            KeyCode::Char('k') | KeyCode::Up => self.splash.move_up(),
            KeyCode::Enter => self.hold_from_splash(self.splash.selected()),
            _ => {},
        }
        Ok(())
    }

    /// Holds the chosen profile where atrium was started, which is what a bare
    /// `atrium` used to do without asking.
    fn hold_from_splash(&mut self, index: usize) {
        let Some(profile) = self.profiles.get(index).cloned() else {
            return;
        };
        let cwd = std::env::current_dir().unwrap_or_else(|_| ".".into());

        match Agent::spawn(&AgentSpec::from_profile(&profile, cwd), &self.harness, &self.theme, self.stage.height, self.stage.width) {
            Ok(agent) => self.registry.push(agent),
            Err(error) => self.splash.error = Some(first_line(&error.to_string())),
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
            Action::Settings => self.settings = Some(Settings::new()),
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
