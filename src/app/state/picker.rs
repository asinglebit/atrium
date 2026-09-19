use crate::core::{profile::Profile, projects::Project};

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

/// Choosing what to hold next: which project, and which profile to hold it
/// under. The profile is the CLI *and* whatever that CLI needs to be launched
/// with, which is how a Claude subscription is chosen.
pub struct Picker {
    projects: Vec<Project>,
    profiles: Vec<Profile>,
    filter: String,
    selected: usize,
    profile: usize,
    /// Why the last attempt to hold something failed, if it did.
    error: Option<String>,
}

impl Picker {
    /// Opens on `profile`, which is the configured default, so the common case
    /// is enter and nothing else.
    pub fn new(projects: Vec<Project>, profiles: Vec<Profile>, profile: usize) -> Self {
        let profile = if profile < profiles.len() { profile } else { 0 };
        Self { projects, profiles, filter: String::new(), selected: 0, profile, error: None }
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

    pub fn profile(&self) -> Option<&Profile> {
        self.profiles.get(self.profile)
    }

    /// What the modal writes next to `tab`.
    pub fn profile_label(&self) -> String {
        self.profile().map(Profile::label).unwrap_or_default()
    }

    pub fn cycle_profile(&mut self) {
        self.error = None;
        if !self.profiles.is_empty() {
            self.profile = (self.profile + 1) % self.profiles.len();
        }
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
