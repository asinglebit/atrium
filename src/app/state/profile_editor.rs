use crate::core::profiles_file::{StoredProfile, StoredProfiles};

/// What can be done to a profile once it is selected. Guitar's remote actions,
/// mapped onto what a profile holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    SetDefault,
    Rename,
    EditConfigDir,
    EditArgs,
    Delete,
}

impl Action {
    pub const ALL: [Action; 5] = [Action::SetDefault, Action::Rename, Action::EditConfigDir, Action::EditArgs, Action::Delete];

    pub fn label(self) -> &'static str {
        match self {
            Self::SetDefault => "set as default",
            Self::Rename => "rename",
            Self::EditConfigDir => "edit config dir",
            Self::EditArgs => "edit args",
            Self::Delete => "delete",
        }
    }
}

/// What is being asked for. Adding chains three of these, the way guitar chains
/// a remote's name into its url.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Prompt {
    AddName,
    AddConfigDir { name: String },
    AddArgs { name: String, config_dir: String },
    Rename(usize),
    EditConfigDir(usize),
    EditArgs(usize),
}

impl Prompt {
    /// What the modal is titled, which is also the only place the args
    /// splitting rule gets explained.
    pub fn title(&self) -> &'static str {
        match self {
            Self::AddName | Self::Rename(_) => "name",
            Self::AddConfigDir { .. } | Self::EditConfigDir(_) => "config dir -- ~ and $VAR are kept as written",
            Self::AddArgs { .. } | Self::EditArgs(_) => "args, split on spaces -- leave empty for none",
        }
    }
}

/// Where the profile editor is. One of these is up at a time, over the settings
/// view, and it owns the keyboard while it is.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Step {
    Actions { index: usize, selected: usize },
    Asking { prompt: Prompt, input: String },
    ConfirmDelete { index: usize },
}

/// A change to make to the stored set. Boxed because each one closes over what
/// was typed, and named because the type is otherwise a mouthful.
pub type Change = Box<dyn FnOnce(&mut StoredProfiles) -> Result<(), String>>;

/// What the editor wants done to the file once a step finishes.
pub enum Outcome {
    /// Still going: another step is up.
    Continue,
    /// Nothing more to do; close.
    Close,
    /// Apply this to the stored profiles, save, and close.
    Commit(Change),
}

pub struct Editor {
    pub step: Step,
    /// Kept on the editor rather than on one step, so a refusal from any of
    /// them has somewhere to be shown.
    pub error: Option<String>,
}

impl Editor {
    pub fn add() -> Self {
        Self { step: Step::Asking { prompt: Prompt::AddName, input: String::new() }, error: None }
    }

    pub fn manage(index: usize) -> Self {
        Self { step: Step::Actions { index, selected: 0 }, error: None }
    }

    /// Opens a prompt already filled in with what is there now, so editing is
    /// a correction rather than retyping.
    fn ask(&mut self, prompt: Prompt, input: String) {
        self.step = Step::Asking { prompt, input };
        self.error = None;
    }

    pub fn name_of(profiles: &StoredProfiles, index: usize) -> String {
        profiles.profiles.get(index).map(|entry| entry.name.clone()).unwrap_or_default()
    }

    pub fn push(&mut self, c: char) {
        if let Step::Asking { input, .. } = &mut self.step {
            input.push(c);
            self.error = None;
        }
    }

    pub fn backspace(&mut self) {
        if let Step::Asking { input, .. } = &mut self.step {
            input.pop();
            self.error = None;
        }
    }

    pub fn move_down(&mut self) {
        if let Step::Actions { selected, .. } = &mut self.step {
            *selected = (*selected + 1) % Action::ALL.len();
        }
    }

    pub fn move_up(&mut self) {
        if let Step::Actions { selected, .. } = &mut self.step {
            *selected = (*selected + Action::ALL.len() - 1) % Action::ALL.len();
        }
    }

    /// Enter. Moves to the next step, or hands back the change to make.
    pub fn confirm(&mut self, profiles: &StoredProfiles) -> Outcome {
        match self.step.clone() {
            Step::Actions { index, selected } => self.chose(Action::ALL[selected], index, profiles),
            Step::ConfirmDelete { index } => Outcome::Commit(Box::new(move |profiles| {
                profiles.remove(index);
                Ok(())
            })),
            Step::Asking { prompt, input } => self.answered(prompt, input),
        }
    }

    fn chose(&mut self, action: Action, index: usize, profiles: &StoredProfiles) -> Outcome {
        let current = profiles.profiles.get(index);
        match action {
            Action::SetDefault => Outcome::Commit(Box::new(move |profiles| {
                profiles.set_default(index);
                Ok(())
            })),
            Action::Delete => {
                self.step = Step::ConfirmDelete { index };
                Outcome::Continue
            },
            Action::Rename => {
                self.ask(Prompt::Rename(index), current.map(|entry| entry.name.clone()).unwrap_or_default());
                Outcome::Continue
            },
            Action::EditConfigDir => {
                self.ask(Prompt::EditConfigDir(index), current.map(|entry| entry.config_dir.clone()).unwrap_or_default());
                Outcome::Continue
            },
            Action::EditArgs => {
                self.ask(Prompt::EditArgs(index), current.map(|entry| entry.args.join(" ")).unwrap_or_default());
                Outcome::Continue
            },
        }
    }

    fn answered(&mut self, prompt: Prompt, input: String) -> Outcome {
        let value = input.trim().to_owned();
        match prompt {
            Prompt::AddName => {
                if value.is_empty() {
                    self.fail("a profile needs a name");
                    return Outcome::Continue;
                }
                // The directory convention guessed from the name, so the common
                // case is two presses of enter.
                let guess = format!("~/.claude-{value}");
                self.ask(Prompt::AddConfigDir { name: value }, guess);
                Outcome::Continue
            },
            Prompt::AddConfigDir { name } => {
                self.ask(Prompt::AddArgs { name, config_dir: value }, String::new());
                Outcome::Continue
            },
            Prompt::AddArgs { name, config_dir } => {
                Outcome::Commit(Box::new(move |profiles| profiles.add(StoredProfile { name, program: String::new(), config_dir, args: split_args(&value), env: Vec::new() })))
            },
            Prompt::Rename(index) => Outcome::Commit(Box::new(move |profiles| profiles.rename(index, &value))),
            Prompt::EditConfigDir(index) => Outcome::Commit(Box::new(move |profiles| {
                let Some(entry) = profiles.profiles.get_mut(index) else {
                    return Err("no such profile".to_owned());
                };
                entry.config_dir = value;
                Ok(())
            })),
            Prompt::EditArgs(index) => Outcome::Commit(Box::new(move |profiles| {
                let Some(entry) = profiles.profiles.get_mut(index) else {
                    return Err("no such profile".to_owned());
                };
                entry.args = split_args(&value);
                Ok(())
            })),
        }
    }

    /// Keeps a refusal inside the modal, so another answer can be given without
    /// losing where you were -- the picker keeps a failed launch the same way.
    pub fn fail(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
    }

    /// Which profile is being worked on, when one is.
    pub fn index(&self) -> Option<usize> {
        match &self.step {
            Step::Actions { index, .. } | Step::ConfirmDelete { index } => Some(*index),
            Step::Asking { prompt, .. } => prompt.index(),
        }
    }

    /// Esc. Backs out of the whole thing rather than one step, because half an
    /// added profile is not worth keeping.
    pub fn cancel(&self) -> Outcome {
        Outcome::Close
    }
}

impl Prompt {
    fn index(&self) -> Option<usize> {
        match self {
            Self::Rename(index) | Self::EditConfigDir(index) | Self::EditArgs(index) => Some(*index),
            Self::AddName | Self::AddConfigDir { .. } | Self::AddArgs { .. } => None,
        }
    }
}

/// Arguments are typed as one line, which cannot carry a quoted argument with a
/// space in it. Whitespace splitting is the honest limit, and the prompt says so.
fn split_args(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_owned).collect()
}

#[cfg(test)]
#[path = "../../tests/app/state/profile_editor.rs"]
mod tests;
