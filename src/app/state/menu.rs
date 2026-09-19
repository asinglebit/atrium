use ratatui::layout::Rect;

use crate::app::input::keymap::Keymap;

/// What picking an entry does. Worked out when the menu is built, so activating
/// one never has to ask where the click originally landed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    /// Put this agent on the stage.
    Focus(usize),
    /// End this agent and take its row away.
    Dismiss(usize),
    New,
    Sidebar,
    Settings,
    Quit,
    /// A line between groups. Never landed on.
    Separator,
}

impl Action {
    pub fn is_pickable(self) -> bool {
        self != Self::Separator
    }
}

/// One line of the menu: what it says, the chord that does the same thing, and
/// what it does.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Item {
    pub label: String,
    pub chord: String,
    pub action: Action,
}

impl Item {
    fn new(label: &str, chord: String, action: Action) -> Self {
        Self { label: label.to_owned(), chord, action }
    }

    fn bare(label: &str, action: Action) -> Self {
        Self { label: label.to_owned(), chord: String::new(), action }
    }

    fn separator() -> Self {
        Self { label: String::new(), chord: String::new(), action: Action::Separator }
    }
}

/// A menu opened at a point, and where the cursor is in it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Menu {
    pub at: (u16, u16),
    pub items: Vec<Item>,
    pub selected: usize,
}

/// Padding and border either side of a label, which is what the box is wider
/// than its longest line by.
const CHROME: u16 = 8;

impl Menu {
    /// The entries for a right-click on an agent's row: what can be done to
    /// that agent, then what can be done regardless.
    pub fn for_agent(at: (u16, u16), index: usize, name: &str, keymap: &Keymap) -> Self {
        let mut items = vec![Item::bare(&format!("go to {name}"), Action::Focus(index)), Item::new(&format!("close {name}"), keymap.dismiss.label(), Action::Dismiss(index)), Item::separator()];
        items.extend(common(keymap));
        Self::new(at, items)
    }

    /// The entries for a right-click anywhere else. `focused` is the agent the
    /// close entry would end, if there is one.
    pub fn general(at: (u16, u16), focused: Option<(usize, &str)>, keymap: &Keymap) -> Self {
        let mut items = Vec::new();
        if let Some((index, name)) = focused {
            items.push(Item::new(&format!("close {name}"), keymap.dismiss.label(), Action::Dismiss(index)));
            items.push(Item::separator());
        }
        items.extend(common(keymap));
        Self::new(at, items)
    }

    fn new(at: (u16, u16), items: Vec<Item>) -> Self {
        let selected = items.iter().position(|item| item.action.is_pickable()).unwrap_or(0);
        Self { at, items, selected }
    }

    /// How wide the box has to be to hold its widest line.
    pub fn width(&self) -> u16 {
        let widest = self.items.iter().map(|item| item.label.chars().count() + item.chord.chars().count()).max().unwrap_or(0);
        widest as u16 + CHROME
    }

    pub fn height(&self) -> u16 {
        self.items.len() as u16 + 2
    }

    /// Where the box goes: at the cursor, pulled back inside the frame when it
    /// would otherwise hang off an edge.
    pub fn area(&self, full: Rect) -> Rect {
        let width = self.width().min(full.width);
        let height = self.height().min(full.height);
        let right = full.x + full.width;
        let bottom = full.y + full.height;

        let x = self.at.0.min(right.saturating_sub(width)).max(full.x);
        let y = self.at.1.min(bottom.saturating_sub(height)).max(full.y);
        Rect { x, y, width, height }
    }

    /// Which entry a click lands on, skipping the border and any separator.
    pub fn item_at(&self, full: Rect, column: u16, row: u16) -> Option<usize> {
        let area = self.area(full);
        if column < area.x || column >= area.x + area.width {
            return None;
        }
        let index = usize::from(row.checked_sub(area.y + 1)?);
        self.items.get(index).filter(|item| item.action.is_pickable()).map(|_| index)
    }

    pub fn covers(&self, full: Rect, column: u16, row: u16) -> bool {
        let area = self.area(full);
        column >= area.x && column < area.x + area.width && row >= area.y && row < area.y + area.height
    }

    pub fn move_down(&mut self) {
        self.step(1);
    }

    pub fn move_up(&mut self) {
        self.step(-1);
    }

    /// Walks over separators rather than stopping on one, and wraps at the ends.
    fn step(&mut self, delta: isize) {
        let len = self.items.len();
        if len == 0 {
            return;
        }
        for hop in 1..=len {
            let at = (self.selected as isize + delta * hop as isize).rem_euclid(len as isize) as usize;
            if self.items[at].action.is_pickable() {
                self.selected = at;
                return;
            }
        }
    }

    pub fn picked(&self) -> Option<Action> {
        self.items.get(self.selected).map(|item| item.action).filter(|action| action.is_pickable())
    }

    pub fn select(&mut self, index: usize) {
        if self.items.get(index).is_some_and(|item| item.action.is_pickable()) {
            self.selected = index;
        }
    }
}

/// The entries every menu ends with, whatever was clicked.
fn common(keymap: &Keymap) -> Vec<Item> {
    vec![
        Item::new("new agent", keymap.new.label(), Action::New),
        Item::new("show / hide sidebar", keymap.sidebar.label(), Action::Sidebar),
        Item::new("settings", keymap.settings.label(), Action::Settings),
        Item::separator(),
        Item::new("quit", keymap.quit.label(), Action::Quit),
    ]
}

#[cfg(test)]
#[path = "../../tests/app/state/menu.rs"]
mod tests;
