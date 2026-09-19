use crate::helpers::scroll;

/// The settings sections, in the order the tab bar shows them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tab {
    General,
    Display,
    Profiles,
    Shortcuts,
}

impl Tab {
    pub const ALL: [Tab; 4] = [Tab::General, Tab::Display, Tab::Profiles, Tab::Shortcuts];

    pub fn label(self) -> &'static str {
        match self {
            Self::General => "general",
            Self::Display => "display",
            Self::Profiles => "profiles",
            Self::Shortcuts => "shortcuts",
        }
    }
}

/// Which way the cursor was last asked to move. A line it cannot land on hands
/// it on *in that direction*, so walking through a section does not bounce back
/// off a heading.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    Up,
    Down,
}

/// What enter does on a line. `Info` rows can be landed on but do nothing,
/// which is what keeps the cursor moving evenly through a list of paths.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SelectionKind {
    Info,
    Theme(usize),
    Profile(usize),
    AddProfile,
    Chord(usize),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Selection {
    pub line: usize,
    pub kind: SelectionKind,
}

/// Where a tab's label landed, recorded while drawing so a click is measured
/// against what was actually painted rather than against the geometry again.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TabHitbox {
    pub tab: Tab,
    pub line: usize,
    pub start: u16,
    pub end: u16,
}

/// Which section is open and where the cursor is in it.
///
/// The cursor is a **line**, not a row. Headings and blanks are lines too, and
/// addressing lines is what lets the cursor be nudged and then snapped onto
/// something selectable, which is how guitar's settings move.
pub struct Settings {
    tab: usize,
    pub selected: usize,
    pub scroll: usize,
    pub last_direction: Option<Direction>,
    /// Filled by each draw: every line that can be landed on, in order.
    pub selections: Vec<Selection>,
    /// Filled by each draw: where the tab labels ended up.
    pub tab_hitboxes: Vec<TabHitbox>,
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

impl Settings {
    pub fn new() -> Self {
        Self { tab: 0, selected: 0, scroll: 0, last_direction: None, selections: Vec::new(), tab_hitboxes: Vec::new() }
    }

    pub fn tab(&self) -> Tab {
        Tab::ALL[self.tab]
    }

    pub fn open(&mut self, tab: Tab) {
        if let Some(index) = Tab::ALL.iter().position(|candidate| *candidate == tab)
            && index != self.tab
        {
            self.tab = index;
            self.restart();
        }
    }

    pub fn next_tab(&mut self) {
        self.tab = (self.tab + 1) % Tab::ALL.len();
        self.restart();
    }

    pub fn previous_tab(&mut self) {
        self.tab = (self.tab + Tab::ALL.len() - 1) % Tab::ALL.len();
        self.restart();
    }

    /// A new section starts at its top. The cursor goes to line zero and the
    /// next draw snaps it down onto the first row worth landing on.
    fn restart(&mut self) {
        self.selected = 0;
        self.scroll = 0;
        self.last_direction = Some(Direction::Down);
        self.selections.clear();
    }

    pub fn move_down(&mut self) {
        self.selected = self.selected.saturating_add(1);
        self.last_direction = Some(Direction::Down);
    }

    pub fn move_up(&mut self) {
        self.selected = self.selected.saturating_sub(1);
        self.last_direction = Some(Direction::Up);
    }

    /// Puts the cursor on a line under the pointer, if that line is one that
    /// can be landed on. A click on a heading is ignored rather than snapped,
    /// because the pointer said exactly where it meant.
    pub fn select_line(&mut self, line: usize) {
        if self.selections.iter().any(|selection| selection.line == line) {
            self.selected = line;
            self.last_direction = None;
        }
    }

    pub fn kind_at_cursor(&self) -> Option<&SelectionKind> {
        self.selections.iter().find(|selection| selection.line == self.selected).map(|selection| &selection.kind)
    }

    /// Lands the cursor on something selectable: the next one the way it was
    /// going, and failing that the nearest by distance. Without this the cursor
    /// would stick on the blank line above a heading.
    pub fn snap(&mut self) {
        if self.selections.is_empty() || self.selections.iter().any(|selection| selection.line == self.selected) {
            return;
        }

        let mut nearest = match self.last_direction {
            Some(Direction::Down) => self.selections.iter().map(|selection| selection.line).find(|line| *line > self.selected),
            Some(Direction::Up) => self.selections.iter().rev().map(|selection| selection.line).find(|line| *line < self.selected),
            None => None,
        };
        if nearest.is_none() {
            nearest = self.selections.iter().map(|selection| selection.line).min_by_key(|line| line.abs_diff(self.selected));
        }
        if let Some(line) = nearest {
            self.selected = line;
        }
    }

    /// Keeps the cursor on screen. The whole view scrolls, header included, so
    /// this counts lines.
    pub fn trap_scroll(&mut self, lines: usize, visible: usize) {
        self.scroll = scroll::trap(self.selected, self.scroll, lines, visible);
    }

    pub fn tab_at(&self, line: usize, column: u16) -> Option<Tab> {
        self.tab_hitboxes.iter().find(|hitbox| hitbox.line == line && column >= hitbox.start && column < hitbox.end).map(|hitbox| hitbox.tab)
    }
}

#[cfg(test)]
#[path = "../../tests/app/state/settings.rs"]
mod tests;
