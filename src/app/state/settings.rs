use crate::helpers::{
    palette::{THEME_PRESETS, Theme},
    scroll,
};

/// The settings view's sections, in the order the tab row shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    Shortcuts,
    Themes,
}

impl Tab {
    pub const ALL: [Tab; 2] = [Tab::Shortcuts, Tab::Themes];

    pub fn label(self) -> &'static str {
        match self {
            Self::Shortcuts => "shortcuts",
            Self::Themes => "themes",
        }
    }
}

/// Which section is open, and where the cursor is in it. One cursor per tab, so
/// switching back and forth does not lose your place.
pub struct Settings {
    tab: usize,
    selected: [usize; Tab::ALL.len()],
    pub scroll: usize,
    /// Where the last draw put each selectable row, so a click maps back to one
    /// without the body having to be built a second time.
    pub row_lines: Vec<usize>,
    /// Where the last draw put the tab bar, for the same reason.
    pub tab_line: usize,
}

impl Settings {
    /// Opens on the theme already in use, so the list starts where you are.
    pub fn new(theme: &Theme) -> Self {
        let at = THEME_PRESETS.iter().position(|preset| preset.theme.name == theme.name).unwrap_or(0);
        Self { tab: 0, selected: [0, at], scroll: 0, row_lines: Vec::new(), tab_line: 0 }
    }

    /// Keeps the selected row on screen. The whole view scrolls, logo included,
    /// so this counts lines rather than rows.
    pub fn trap_scroll(&mut self, lines: usize, visible: usize) {
        let line = self.row_lines.get(self.selected()).copied().unwrap_or(0);
        self.scroll = scroll::trap(line, self.scroll, lines, visible);
    }

    pub fn tab(&self) -> Tab {
        Tab::ALL[self.tab]
    }

    pub fn open(&mut self, tab: Tab) {
        if let Some(index) = Tab::ALL.iter().position(|candidate| *candidate == tab) {
            self.tab = index;
            self.scroll = 0;
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % Tab::ALL.len();
        self.scroll = 0;
    }

    pub fn previous_tab(&mut self) {
        self.tab = (self.tab + Tab::ALL.len() - 1) % Tab::ALL.len();
        self.scroll = 0;
    }

    pub fn selected(&self) -> usize {
        self.selected[self.tab]
    }

    pub fn select(&mut self, index: usize) {
        if index < self.len() {
            self.selected[self.tab] = index;
        }
    }

    pub fn len(&self) -> usize {
        match self.tab() {
            Tab::Shortcuts => 8,
            Tab::Themes => THEME_PRESETS.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn move_down(&mut self) {
        let len = self.len();
        if len > 0 {
            self.selected[self.tab] = (self.selected() + 1) % len;
        }
    }

    pub fn move_up(&mut self) {
        let len = self.len();
        if len > 0 {
            self.selected[self.tab] = (self.selected() + len - 1) % len;
        }
    }

    /// The theme the cursor is on, when the themes tab is the one open.
    pub fn theme_under_cursor(&self) -> Option<Theme> {
        match self.tab() {
            Tab::Themes => THEME_PRESETS.get(self.selected()).map(|preset| preset.theme),
            Tab::Shortcuts => None,
        }
    }
}

#[cfg(test)]
#[path = "../../tests/app/state/settings.rs"]
mod tests;
