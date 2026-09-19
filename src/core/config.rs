use std::path::PathBuf;

use crate::{
    app::input::keymap::{self, Keymap},
    helpers::palette::{self, Theme},
};

/// Everything `~/.config/atrium/config.toml` can say.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Config {
    pub theme: Theme,
    pub keymap: Keymap,
    /// What the file got wrong. Kept rather than discarded so `--check-config`
    /// can say so; the TUI itself carries on with the defaults.
    pub problems: Vec<String>,
}

impl Config {
    pub fn path() -> PathBuf {
        config_home().join("atrium").join("config.toml")
    }

    /// A missing file is not a problem -- it is the normal case.
    pub fn load() -> Self {
        match std::fs::read_to_string(Self::path()) {
            Ok(text) => Self::parse(&text),
            Err(_) => Self::default(),
        }
    }

    pub fn parse(text: &str) -> Self {
        let mut config = Self::default();

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
        config
    }

    /// `name` picks the preset and every other key overrides it, so the preset
    /// has to be applied first whatever order the file is written in.
    fn read_theme(&mut self, section: &toml::Table) {
        if let Some(value) = section.get("name") {
            match value.as_str().and_then(Theme::preset) {
                Some(theme) => self.theme = theme,
                None => self.problems.push(format!("theme.name: unknown theme {value}, expected one of {}", palette::PRESETS.join(", "))),
            }
        }

        for (key, value) in section {
            if key == "name" {
                continue;
            }
            let Some(text) = value.as_str() else {
                self.problems.push(format!("theme.{key}: expected a colour as a string, got {value}"));
                continue;
            };
            let Some(color) = palette::parse_color(text) else {
                self.problems.push(format!("theme.{key}: {text:?} is not a colour -- use #rrggbb or a colour name"));
                continue;
            };
            if !self.theme.set(key, color) {
                self.problems.push(format!("theme.{key}: no such colour to set"));
            }
        }
    }

    fn read_keys(&mut self, section: &toml::Table) {
        for (action, value) in section {
            let Some(text) = value.as_str() else {
                self.problems.push(format!("keys.{action}: expected a key as a string, got {value}"));
                continue;
            };
            let Some(key) = keymap::parse_key(text) else {
                self.problems.push(format!("keys.{action}: {text:?} is not a key"));
                continue;
            };
            if !self.keymap.set(action, key) {
                self.problems.push(format!("keys.{action}: no such action"));
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
