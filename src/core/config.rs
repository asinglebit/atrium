use std::path::PathBuf;

use crate::{
    app::input::keymap::{self, Keymap},
    core::profile::{self, Profile},
    helpers::palette::{self, Theme},
};

/// Everything `~/.config/atrium/config.toml` can say.
#[derive(Clone, Debug)]
pub struct Config {
    pub theme: Theme,
    pub keymap: Keymap,
    /// What can be held, in the order the picker cycles them.
    pub profiles: Vec<Profile>,
    /// Which of them a bare `atrium` uses.
    pub default_profile: usize,
    /// What the file got wrong. Kept rather than discarded so `--check-config`
    /// can say so; the TUI itself carries on with the defaults.
    pub problems: Vec<String>,
}

impl Default for Config {
    /// A fixed base, so parsing is the same everywhere. `load` swaps in
    /// whatever theme.json says before the file gets a look at it.
    fn default() -> Self {
        Self { theme: Theme::classic(), keymap: Keymap::default(), profiles: profile::defaults(), default_profile: 0, problems: Vec::new() }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        config_home().join("atrium").join("config.toml")
    }

    /// A missing file is not a problem -- it is the normal case.
    pub fn load() -> Self {
        let base = palette::load_theme();
        match std::fs::read_to_string(Self::path()) {
            Ok(text) => Self::parse_with(&text, base),
            Err(_) => Self { theme: base, ..Self::default() },
        }
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
        if let Some(value) = table.get("profiles") {
            config.read_profiles(value);
        }
        // Read last, so it can name a profile the same file defined.
        if let Some(value) = table.get("default") {
            config.read_default(value);
        }
        config
    }

    /// Which profile a bare `atrium` holds. Falls back to the first one.
    pub fn default_profile(&self) -> &Profile {
        &self.profiles[self.default_profile.min(self.profiles.len() - 1)]
    }

    pub fn profile_named(&self, name: &str) -> Option<&Profile> {
        self.profiles.iter().find(|profile| profile.name == name)
    }

    fn read_profiles(&mut self, value: &toml::Value) {
        let Some(entries) = value.as_array() else {
            self.problems.push(format!("profiles: expected [[profiles]] blocks, got {value}"));
            return;
        };

        let mut profiles = Vec::new();
        for (index, entry) in entries.iter().enumerate() {
            match entry.as_table() {
                Some(table) => profiles.extend(self.read_profile(index, table)),
                None => self.problems.push(format!("profiles[{index}]: expected a [[profiles]] block, got {entry}")),
            }
        }

        // Nothing usable: keep the built-in list rather than offering nothing at
        // all, which would leave the picker with no way to hold anything.
        if !profiles.is_empty() {
            self.profiles = profiles;
        }
    }

    fn read_profile(&mut self, index: usize, table: &toml::Table) -> Option<Profile> {
        const SETTINGS: [&str; 5] = ["name", "program", "args", "config_dir", "env"];

        let name = match table.get("name").and_then(toml::Value::as_str).map(str::trim) {
            Some(name) if !name.is_empty() => name.to_owned(),
            _ => {
                self.problems.push(format!("profiles[{index}]: needs a `name`"));
                return None;
            },
        };
        let program = table.get("program").and_then(toml::Value::as_str).unwrap_or(profile::DEFAULT_PROGRAM).to_owned();

        let mut args = Vec::new();
        if let Some(value) = table.get("args") {
            match value.as_array() {
                Some(items) => {
                    for item in items {
                        match item.as_str() {
                            Some(text) => args.push(profile::expand(text)),
                            None => self.problems.push(format!("profiles.{name}.args: expected strings, got {item}")),
                        }
                    }
                },
                None => self.problems.push(format!("profiles.{name}.args: expected a list of strings, got {value}")),
            }
        }

        let mut env = Vec::new();
        if let Some(value) = table.get("config_dir") {
            match value.as_str() {
                Some(dir) => env.push((profile::CONFIG_DIR_ENV.to_owned(), profile::expand(dir))),
                None => self.problems.push(format!("profiles.{name}.config_dir: expected a path as a string, got {value}")),
            }
        }
        if let Some(value) = table.get("env") {
            match value.as_table() {
                Some(vars) => {
                    for (key, item) in vars {
                        match item.as_str() {
                            Some(text) => env.push((key.clone(), profile::expand(text))),
                            None => self.problems.push(format!("profiles.{name}.env.{key}: expected a string, got {item}")),
                        }
                    }
                },
                None => self.problems.push(format!("profiles.{name}.env: expected a table, got {value}")),
            }
        }

        for key in table.keys() {
            if !SETTINGS.contains(&key.as_str()) {
                self.problems.push(format!("profiles.{name}.{key}: no such setting"));
            }
        }

        Some(Profile { name, program, args, env })
    }

    fn read_default(&mut self, value: &toml::Value) {
        let Some(name) = value.as_str() else {
            self.problems.push(format!("default: expected a profile name as a string, got {value}"));
            return;
        };
        match self.profiles.iter().position(|profile| profile.name == name) {
            Some(index) => self.default_profile = index,
            None => self.problems.push(format!("default: no profile named {name:?} -- there is {}", self.profiles.iter().map(|profile| profile.name.as_str()).collect::<Vec<_>>().join(", "))),
        }
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
