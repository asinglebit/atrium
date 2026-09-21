use crate::core::{
    profile,
    profiles_file::{StoredProfile, StoredProfiles},
};

/// What can be done to a profile once it is selected. Guitar's remote actions,
/// mapped onto what a profile holds.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    SetDefault,
    Rename,
    EditProgram,
    EditConfigDir,
    EditArgs,
    Delete,
}

impl Action {
    pub const ALL: [Action; 6] = [Action::SetDefault, Action::Rename, Action::EditProgram, Action::EditConfigDir, Action::EditArgs, Action::Delete];

    pub fn label(self) -> &'static str {
        match self {
            Self::SetDefault => "set as default",
            Self::Rename => "rename",
            Self::EditProgram => "edit program",
            Self::EditConfigDir => "edit config dir",
            Self::EditArgs => "edit args",
            Self::Delete => "delete",
        }
    }
}

/// What is being asked for. Adding chains these, the way guitar chains a
/// remote's name into its url. The steps after the program carry it along,
/// because which CLI it is decides the directory guess and what the directory
/// is even called.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Prompt {
    AddName,
    AddProgram { name: String },
    AddConfigDir { name: String, program: String },
    AddArgs { name: String, program: String, config_dir: String },
    Rename(usize),
    EditProgram(usize),
    EditConfigDir { index: usize, program: String },
    EditArgs(usize),
}

impl Prompt {
    /// What the modal is titled, which is also the only place the args
    /// splitting rule gets explained, and the only place the directory says
    /// which variable it actually sets.
    pub fn title(&self) -> String {
        match self {
            Self::AddName | Self::Rename(_) => "name".to_owned(),
            Self::AddProgram { .. } | Self::EditProgram(_) => format!("program -- leave empty for {}", profile::DEFAULT_PROGRAM),
            Self::AddConfigDir { program, .. } | Self::EditConfigDir { program, .. } => match profile::config_dir_env(program) {
                Some(key) => format!("config dir ({key}) -- ~ and $VAR are kept as written"),
                None => "config dir -- ~ and $VAR are kept as written".to_owned(),
            },
            Self::AddArgs { .. } | Self::EditArgs(_) => "args, split on spaces -- leave empty for none".to_owned(),
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
            Action::EditProgram => {
                self.ask(Prompt::EditProgram(index), current.map(|entry| entry.program.clone()).unwrap_or_default());
                Outcome::Continue
            },
            Action::EditConfigDir => {
                let program = current.map_or_else(|| profile::DEFAULT_PROGRAM.to_owned(), |entry| entry.program_or_default().to_owned());
                // A CLI with no directory of its own is told so rather than
                // handed a prompt whose answer nothing would ever read.
                if profile::config_dir_env(&program).is_none() {
                    self.fail(format!("{program} keeps no config directory atrium can set"));
                    return Outcome::Continue;
                }
                self.ask(Prompt::EditConfigDir { index, program }, current.map(|entry| entry.config_dir.clone()).unwrap_or_default());
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
                self.ask(Prompt::AddProgram { name: value }, profile::DEFAULT_PROGRAM.to_owned());
                Outcome::Continue
            },
            Prompt::AddProgram { name } => {
                let program = if value.is_empty() { profile::DEFAULT_PROGRAM.to_owned() } else { value };
                // A CLI that keeps no directory of its own skips straight past
                // the question, rather than being asked one it has no answer to.
                match profile::config_dir_guess(&program, &name) {
                    // The directory convention guessed from the name, so the
                    // common case is one more press of enter.
                    Some(guess) => self.ask(Prompt::AddConfigDir { name, program }, guess),
                    None => self.ask(Prompt::AddArgs { name, program, config_dir: String::new() }, String::new()),
                }
                Outcome::Continue
            },
            Prompt::AddConfigDir { name, program } => {
                self.ask(Prompt::AddArgs { name, program, config_dir: value }, String::new());
                Outcome::Continue
            },
            Prompt::AddArgs { name, program, config_dir } => {
                Outcome::Commit(Box::new(move |profiles| profiles.add(StoredProfile { name, program: stored_program(&program), config_dir, args: split_args(&value), env: Vec::new() })))
            },
            Prompt::Rename(index) => Outcome::Commit(Box::new(move |profiles| profiles.rename(index, &value))),
            Prompt::EditProgram(index) => Outcome::Commit(Box::new(move |profiles| {
                let Some(entry) = profiles.profiles.get_mut(index) else {
                    return Err("no such profile".to_owned());
                };
                entry.program = stored_program(&value);
                Ok(())
            })),
            Prompt::EditConfigDir { index, .. } => Outcome::Commit(Box::new(move |profiles| {
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
            Self::Rename(index) | Self::EditProgram(index) | Self::EditConfigDir { index, .. } | Self::EditArgs(index) => Some(*index),
            Self::AddName | Self::AddProgram { .. } | Self::AddConfigDir { .. } | Self::AddArgs { .. } => None,
        }
    }
}

/// The default CLI is stored as nothing, which is what an unwritten `program`
/// field already means -- so a profile does not start naming `claude` just
/// because the prompt showed it.
fn stored_program(value: &str) -> String {
    if value.trim() == profile::DEFAULT_PROGRAM { String::new() } else { value.trim().to_owned() }
}

/// Arguments are typed as one line, which cannot carry a quoted argument with a
/// space in it. Whitespace splitting is the honest limit, and the prompt says so.
fn split_args(value: &str) -> Vec<String> {
    value.split_whitespace().map(str::to_owned).collect()
}

#[cfg(test)]
#[path = "../../tests/app/state/profile_editor.rs"]
mod tests;
