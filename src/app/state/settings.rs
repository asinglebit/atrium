use crate::helpers::palette::{THEME_PRESETS, Theme};

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
}

impl Settings {
    /// Opens on the theme already in use, so the list starts where you are.
    pub fn new(theme: &Theme) -> Self {
        let at = THEME_PRESETS.iter().position(|preset| preset.theme.name == theme.name).unwrap_or(0);
        Self { tab: 0, selected: [0, at], scroll: 0 }
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
