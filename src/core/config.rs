use std::path::PathBuf;

use crate::{
    app::input::keymap::{self, Keymap},
    core::{
        installed::{self, Found},
        profile::Profile,
        profiles_file::{self, StoredProfiles},
    },
    helpers::palette::{self, Theme},
};

/// Everything `~/.config/atrium/config.toml` can say.
#[derive(Clone, Debug)]
pub struct Config {
    pub theme: Theme,
    pub keymap: Keymap,
    /// What can be held, in the order the picker cycles them, with every path
    /// and argument expanded ready to launch.
    pub profiles: Vec<Profile>,
    /// Which of them a bare `atrium` uses.
    pub default_profile: usize,
    /// The same profiles as they are written down, which is what the settings
    /// view shows and edits. Raw, so saving cannot hardcode a home directory.
    pub stored_profiles: StoredProfiles,
    /// Which of the CLIs atrium knows are on this machine. What is installed
    /// and not already spoken for is offered alongside the profiles.
    pub installed: Vec<Found>,
    /// What the file got wrong. Kept rather than discarded so `--check-config`
    /// can say so; the TUI itself carries on with the defaults.
    pub problems: Vec<String>,
}

impl Default for Config {
    /// A fixed base, so parsing is the same everywhere. `load` swaps in
    /// whatever theme.json says before the file gets a look at it.
    fn default() -> Self {
        Self { theme: Theme::classic(), keymap: Keymap::default(), profiles: Vec::new(), default_profile: 0, stored_profiles: StoredProfiles::default(), installed: Vec::new(), problems: Vec::new() }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        config_home().join("atrium").join("config.toml")
    }

    /// A missing file is not a problem -- it is the normal case.
    pub fn load() -> Self {
        let base = palette::load_theme();
        let mut config = match std::fs::read_to_string(Self::path()) {
            Ok(text) => Self::parse_with(&text, base),
            Err(_) => Self { theme: base, ..Self::default() },
        };
        config.installed = installed::scan();
        config.adopt(profiles_file::load());
        config
    }

    /// Takes what `profiles.json` says, against whatever the `PATH` scan found.
    /// Split out from `load` so parsing a config file stays off the disk, which
    /// is what keeps the tests hermetic.
    pub fn adopt(&mut self, stored: StoredProfiles) {
        let (profiles, default_profile) = stored.resolve(&self.installed);
        self.profiles = profiles;
        self.default_profile = default_profile;
        self.stored_profiles = stored;
    }

    /// Parses against a fixed base theme, which is what keeps tests off the
    /// machine's own theme.json.
    pub fn parse(text: &str) -> Self {
        Self::parse_with(text, Theme::classic())
    }

    pub fn parse_with(text: &str, base: Theme) -> Self {
        let mut config = Self { theme: base, ..Self::default() };

        let table: toml::Table = match text.parse() {
            Ok(table) => table,
            Err(error) => {
                config.problems.push(format!("not valid TOML: {error}"));
                return config;
            },
        };

        if let Some(section) = table.get("theme").and_then(toml::Value::as_table) {
            config.read_theme(section);
        }
        if let Some(section) = table.get("keys").and_then(toml::Value::as_table) {
            config.read_keys(section);
        }
        // Profiles moved to a file atrium writes, because they are edited in the
        // settings view now and rewriting this one would lose your comments.
        for moved in ["profiles", "default"] {
            if table.contains_key(moved) {
                config.problems.push(format!("{moved}: profiles live in {} now, and are edited in settings -- this is ignored", profiles_file::path().display()));
            }
        }
        config
    }

    /// Which profile a bare `atrium` holds. Falls back to the first one, and to
    /// none at all on a machine where nothing atrium knows is installed.
    pub fn default_profile(&self) -> Option<&Profile> {
        self.profiles.get(self.default_profile).or_else(|| self.profiles.first())
    }

    pub fn profile_named(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.name == name)
    }

    /// Only `name` lives here. Individual colours are guitar's `theme.json`
    /// format, shared with guitar so the two never drift apart.
    fn read_theme(&mut self, section: &toml::Table) {
        for (key, value) in section {
            if key != "name" {
                self.problems.push(format!("theme.{key}: only `name` belongs here -- individual colours live in theme.json, shared with guitar"));
                continue;
            }
            let Some(label) = value.as_str() else {
                self.problems.push(format!("theme.name: expected a theme name as a string, got {value}"));
                continue;
            };
            match palette::preset_named(label) {
                Some(theme) => self.theme = theme,
                None => self.problems.push(format!("theme.name: unknown theme {label:?}")),
            }
        }
    }

    fn read_keys(&mut self, section: &toml::Table) {
        for (action, value) in section {
            let Some(text) = value.as_str() else {
                self.problems.push(format!("keys.{action}: expected a key as a string, got {value}"));
                continue;
            };
            let Some(chord) = keymap::parse_chord(text) else {
                self.problems.push(format!("keys.{action}: {text:?} is not a key -- try q, f12, ctrl+g, alt+enter"));
                continue;
            };
            if !self.keymap.set(action, chord) {
                let hint = if action == "leader" { " -- there is no leader any more; actions fire directly, so bind them one by one" } else { "" };
                self.problems.push(format!("keys.{action}: no such action{hint}"));
            }
        }
    }
}

/// `$XDG_CONFIG_HOME`, else `~/.config`, matching what every other tool here does.
fn config_home() -> PathBuf {
    if let Some(dir) = std::env::var_os("XDG_CONFIG_HOME") {
        return PathBuf::from(dir);
    }
    std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), |home| PathBuf::from(home).join(".config"))
}

#[cfg(test)]
#[path = "../tests/core/config.rs"]
mod tests;
