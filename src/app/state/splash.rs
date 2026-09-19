/// Where the cursor is on the splash. The list is what can be held, so the
/// cursor is an index into the profiles.
pub struct Splash {
    selected: usize,
    len: usize,
    /// Why the last attempt to hold something failed, if it did. The picker
    /// keeps a failed launch the same way.
    pub error: Option<String>,
}

impl Splash {
    /// Opens on the default profile, so the common case is one press of enter.
    pub fn new(len: usize, default: usize) -> Self {
        Self { selected: if default < len { default } else { 0 }, len, error: None }
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn move_down(&mut self) {
        self.error = None;
        if self.len > 0 {
            self.selected = (self.selected + 1) % self.len;
        }
    }

    pub fn move_up(&mut self) {
        self.error = None;
        if self.len > 0 {
            self.selected = (self.selected + self.len - 1) % self.len;
        }
    }

    /// Which row a click landed on, given where the list was drawn.
    pub fn row_at(&self, first: u16, row: u16) -> Option<usize> {
        let index = usize::from(row.checked_sub(first)?);
        (index < self.len).then_some(index)
    }
}

#[cfg(test)]
#[path = "../../tests/app/state/splash.rs"]
mod tests;
