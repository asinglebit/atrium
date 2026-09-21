use std::{
    fs,
    path::{Path, PathBuf},
};

use facet::Facet;

use crate::core::{
    installed::Found,
    profile::{self, Profile},
};

/// One environment variable, as a pair of named fields rather than a tuple so
/// that it reads as itself in the file.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct StoredVar {
    pub name: String,
    pub value: String,
}

/// A profile the way it is written down, which is **not** the way it is used.
///
/// Everything here is stored raw: `~/.claude-work` stays `~/.claude-work` on
/// disk and is expanded only on the way to a child process. Saving an expanded
/// path would quietly hardcode one machine's home directory into the file.
#[derive(Facet, Clone, Debug, PartialEq, Eq)]
pub struct StoredProfile {
    pub name: String,
    /// Empty means `claude`, which is what a subscription is.
    #[facet(default)]
    pub program: String,
    /// Shorthand for whichever variable this CLI keeps its configuration under
    /// -- `CLAUDE_CONFIG_DIR`, `COPILOT_HOME`. Empty means the profile sets none.
    #[facet(default)]
    pub config_dir: String,
    #[facet(default)]
    pub args: Vec<String>,
    #[facet(default)]
    pub env: Vec<StoredVar>,
}

impl StoredProfile {
    /// A profile named after a subscription, guessing the directory that
    /// convention would put it in.
    pub fn named(name: &str) -> Self {
        Self::named_for(profile::DEFAULT_PROGRAM, name)
    }

    /// The same, for a CLI other than the default one. A CLI with no directory
    /// of its own to name gets none rather than a guess.
    pub fn named_for(program: &str, name: &str) -> Self {
        let config_dir = profile::config_dir_guess(program, name).unwrap_or_default();
        // The default one is stored as nothing, which is what an unwritten
        // `program` field already means.
        let program = if program == profile::DEFAULT_PROGRAM { String::new() } else { program.to_owned() };
        Self { name: name.to_owned(), program, config_dir, args: Vec::new(), env: Vec::new() }
    }

    pub fn program_or_default(&self) -> &str {
        if self.program.trim().is_empty() { profile::DEFAULT_PROGRAM } else { &self.program }
    }

    /// The runnable form: every path and argument expanded.
    pub fn resolve(&self) -> Profile {
        let mut env = Vec::new();
        // A directory typed against a CLI with no variable for one is left
        // where it was written rather than turned into an environment variable
        // nothing reads.
        if let Some(key) = profile::config_dir_env(self.program_or_default())
            && !self.config_dir.trim().is_empty()
        {
            env.push((key.to_owned(), profile::expand(&self.config_dir)));
        }
        env.extend(self.env.iter().map(|var| (var.name.clone(), profile::expand(&var.value))));

        Profile { name: self.name.clone(), program: self.program_or_default().to_owned(), args: self.args.iter().map(|arg| profile::expand(arg)).collect(), env }
    }
}

/// The whole file: what can be held, and which of them a bare `atrium` holds.
#[derive(Facet, Clone, Debug, PartialEq, Eq, Default)]
pub struct StoredProfiles {
    #[facet(default)]
    pub default: String,
    #[facet(default)]
    pub profiles: Vec<StoredProfile>,
}

impl StoredProfiles {
    pub fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub fn position(&self, name: &str) -> Option<usize> {
        self.profiles.iter().position(|entry| entry.name == name)
    }

    /// Which entry is the default. Falls back to the first, so there is always
    /// one as long as there is anything at all.
    pub fn default_index(&self) -> usize {
        self.position(&self.default).unwrap_or(0)
    }

    /// The runnable list, and which of it is the default: what was written
    /// down, and then every installed CLI the file did not already speak for.
    ///
    /// The stored ones come first, so the default stays where `default_index`
    /// found it, and so a machine with nothing configured still has something
    /// to offer.
    pub fn resolve(&self, installed: &[Found]) -> (Vec<Profile>, usize) {
        let mut profiles: Vec<Profile> = self.profiles.iter().map(StoredProfile::resolve).collect();
        profiles.extend(installed.iter().filter(|found| !self.holds_program(found.program)).map(|found| Profile::bare(found.program)));
        (profiles, self.default_index())
    }

    /// Whether a profile already launches this CLI. One that does makes the
    /// bare row redundant -- it would offer `claude` beside the two profiles
    /// that are how you actually hold claude.
    fn holds_program(&self, program: &str) -> bool {
        self.profiles.iter().any(|entry| entry.program_or_default() == program)
    }

    /// Adds one, refusing a name already taken -- two profiles with one name
    /// could not be told apart in the picker or on a row.
    pub fn add(&mut self, entry: StoredProfile) -> Result<(), String> {
        if entry.name.trim().is_empty() {
            return Err("a profile needs a name".to_owned());
        }
        if self.position(&entry.name).is_some() {
            return Err(format!("there is already a profile called {:?}", entry.name));
        }
        if self.profiles.is_empty() {
            self.default = entry.name.clone();
        }
        self.profiles.push(entry);
        Ok(())
    }

    pub fn rename(&mut self, index: usize, name: &str) -> Result<(), String> {
        let name = name.trim();
        if name.is_empty() {
            return Err("a profile needs a name".to_owned());
        }
        if self.position(name).is_some_and(|taken| taken != index) {
            return Err(format!("there is already a profile called {name:?}"));
        }
        let Some(entry) = self.profiles.get_mut(index) else {
            return Err("no such profile".to_owned());
        };
        let was_default = self.default == entry.name;
        entry.name = name.to_owned();
        // The default is held by name, so renaming has to carry it along.
        if was_default {
            self.default = name.to_owned();
        }
        Ok(())
    }

    /// Removes one, moving the default off it if that is what it was.
    pub fn remove(&mut self, index: usize) {
        if index >= self.profiles.len() {
            return;
        }
        let removed = self.profiles.remove(index);
        if self.default == removed.name {
            self.default = self.profiles.first().map(|entry| entry.name.clone()).unwrap_or_default();
        }
    }

    pub fn set_default(&mut self, index: usize) {
        if let Some(entry) = self.profiles.get(index) {
            self.default = entry.name.clone();
        }
    }
}

/// Beside `theme.json` and `layout.json`, the other files atrium writes for
/// itself. `config.toml` stays a file only you write.
pub fn path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("atrium");
    path.push("profiles.json");
    path
}

pub fn load() -> StoredProfiles {
    load_from(&path())
}

pub fn save(profiles: &StoredProfiles) -> Result<(), String> {
    save_to(&path(), profiles)
}

/// A file that is missing, unreadable or malformed is not a problem: the
/// built-in list is a perfectly good answer.
pub fn load_from(path: &Path) -> StoredProfiles {
    fs::read_to_string(path).ok().and_then(|text| facet_json::from_str::<StoredProfiles>(&text).ok()).unwrap_or_default()
}

/// Unlike the theme and the layout, this one reports failure: it is written in
/// answer to something you just did in the settings, so silence would look like
/// the edit had worked.
pub fn save_to(path: &Path, profiles: &StoredProfiles) -> Result<(), String> {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        fs::create_dir_all(parent).map_err(|error| format!("could not create {}: {error}", parent.display()))?;
    }
    let text = facet_json::to_string_pretty(profiles).map_err(|error| format!("could not encode profiles: {error}"))?;
    fs::write(path, text).map_err(|error| format!("could not write {}: {error}", path.display()))
}

#[cfg(test)]
#[path = "../tests/core/profiles_file.rs"]
mod tests;
