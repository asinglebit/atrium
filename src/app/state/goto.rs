use crate::app::input::keymap;

/// Choosing which held agent to show next. No filter: the list is short, and
/// the numbers on the rows are the fast path.
pub struct Goto {
    len: usize,
    selected: usize,
}

impl Goto {
    pub fn new(len: usize, focused: usize) -> Self {
        Self { len, selected: focused.min(len.saturating_sub(1)) }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn move_down(&mut self) {
        if self.len > 0 {
            self.selected = (self.selected + 1) % self.len;
        }
    }

    pub fn move_up(&mut self) {
        if self.len > 0 {
            self.selected = (self.selected + self.len - 1) % self.len;
        }
    }

    /// The row a digit names, or None when it names one that is not there.
    /// Numbered the way the prefix numbers them: from one, `0` the tenth.
    pub fn row_for(&self, digit: char) -> Option<usize> {
        let index = keymap::row_for_digit(digit)?;
        (index < self.len).then_some(index)
    }
}

#[cfg(test)]
#[path = "../../tests/app/state/goto.rs"]
mod tests;
