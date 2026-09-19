use crate::core::projects::Project;

/// The agent CLIs the picker can launch, in the order Tab cycles them.
pub const KINDS: [&str; 3] = ["claude", "opencode", "codex"];

/// How well `needle` fits `haystack`, lower being better: how far apart the
/// matched characters are, then how late the match starts, then the name's
/// length. None when the characters do not all appear in order.
///
/// The span is what matters. Matching loosely means "gui" finds both `guitar`
/// and `asinglebit.github.io`; ranking by span is what puts `guitar` first.
fn score(needle: &str, haystack: &str) -> Option<(usize, usize, usize)> {
    if needle.is_empty() {
        // Everything ties, so the name tie-break leaves the list alphabetical.
        return Some((0, 0, 0));
    }

    let hay: Vec<char> = haystack.chars().flat_map(char::to_lowercase).collect();
    let (mut first, mut last, mut from) = (None, 0, 0);
    for wanted in needle.chars().flat_map(char::to_lowercase) {
        let at = from + hay[from..].iter().position(|have| *have == wanted)?;
        first.get_or_insert(at);
        last = at;
        from = at + 1;
    }

    let first = first?;
    Some((last - first, first, hay.len()))
}

/// Choosing what to hold next: which project, and which CLI to hold there.
pub struct Picker {
    projects: Vec<Project>,
    filter: String,
    selected: usize,
    kind: usize,
    /// Why the last attempt to hold something failed, if it did.
    error: Option<String>,
}

impl Picker {
    pub fn new(projects: Vec<Project>) -> Self {
        Self { projects, filter: String::new(), selected: 0, kind: 0, error: None }
    }

    pub fn error(&self) -> Option<&str> {
        self.error.as_deref()
    }

    /// A failed launch keeps the modal open so another choice can be made.
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
    }

    pub fn filter(&self) -> &str {
        &self.filter
    }

    pub fn kind(&self) -> &'static str {
        KINDS[self.kind]
    }

    pub fn cycle_kind(&mut self) {
        self.error = None;
        self.kind = (self.kind + 1) % KINDS.len();
    }

    /// Ranked, not just filtered: the tightest match comes first.
    pub fn matches(&self) -> Vec<&Project> {
        let mut scored: Vec<((usize, usize, usize), &Project)> = self.projects.iter().filter_map(|project| score(&self.filter, &project.name).map(|rank| (rank, project))).collect();
        scored.sort_by(|(left, a), (right, b)| left.cmp(right).then_with(|| a.name.cmp(&b.name)));
        scored.into_iter().map(|(_, project)| project).collect()
    }

    pub fn selected(&self) -> usize {
        self.selected
    }

    pub fn selected_project(&self) -> Option<&Project> {
        self.matches().into_iter().nth(self.selected)
    }

    pub fn move_down(&mut self) {
        let count = self.matches().len();
        if count > 0 {
            self.selected = (self.selected + 1) % count;
        }
    }

    pub fn move_up(&mut self) {
        let count = self.matches().len();
        if count > 0 {
            self.selected = (self.selected + count - 1) % count;
        }
    }

    /// Typing restarts the selection at the top, because the list underneath it
    /// has just changed.
    pub fn push(&mut self, c: char) {
        self.error = None;
        self.filter.push(c);
        self.selected = 0;
    }

    pub fn backspace(&mut self) {
        self.error = None;
        self.filter.pop();
        self.selected = 0;
    }
}

#[cfg(test)]
#[path = "../../tests/app/state/picker.rs"]
mod tests;
