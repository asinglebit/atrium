use ratatui::style::Color;

use crate::core::agent::Status;

/// atrium's colours are named for what they mean, not for what they are. There
/// are only a handful of coloured things here, so a role per thing beats
/// carrying a general-purpose palette around.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub border: Color,
    pub text: Color,
    pub dim: Color,
    pub idle: Color,
    pub working: Color,
    pub needs_input: Color,
    pub error: Color,
    pub exited: Color,
}

/// The presets `[theme] name` accepts, in the order `--help` lists them.
pub const PRESETS: [&str; 3] = ["greyscale", "classic", "ansi"];

impl Default for Theme {
    fn default() -> Self {
        Self::greyscale()
    }
}

impl Theme {
    /// The default, tuned to the greyscale desktop this was written on: colour
    /// only where it carries meaning, everything else in grey.
    pub fn greyscale() -> Self {
        Self {
            border: Color::Rgb(66, 66, 66),
            text: Color::Rgb(216, 216, 216),
            dim: Color::Rgb(117, 117, 117),
            idle: Color::Rgb(158, 158, 158),
            working: Color::Rgb(189, 189, 189),
            needs_input: Color::Rgb(255, 255, 255),
            error: Color::Rgb(239, 83, 80),
            exited: Color::Rgb(97, 97, 97),
        }
    }

    /// guitar's colours, for when atrium sits next to it.
    pub fn classic() -> Self {
        Self {
            border: Color::Rgb(66, 66, 66),
            text: Color::Rgb(189, 189, 189),
            dim: Color::Rgb(117, 117, 117),
            idle: Color::Rgb(158, 158, 158),
            working: Color::Rgb(255, 238, 88),
            needs_input: Color::Rgb(102, 187, 106),
            error: Color::Rgb(239, 83, 80),
            exited: Color::Rgb(97, 97, 97),
        }
    }

    /// The terminal's own sixteen, so atrium follows whatever the terminal
    /// theme already is.
    pub fn ansi() -> Self {
        Self { border: Color::DarkGray, text: Color::Gray, dim: Color::DarkGray, idle: Color::Gray, working: Color::Yellow, needs_input: Color::Green, error: Color::Red, exited: Color::DarkGray }
    }

    pub fn preset(name: &str) -> Option<Self> {
        match name {
            "greyscale" => Some(Self::greyscale()),
            "classic" => Some(Self::classic()),
            "ansi" => Some(Self::ansi()),
            _ => None,
        }
    }

    pub fn for_status(&self, status: Status) -> Color {
        match status {
            Status::Idle => self.idle,
            Status::Working => self.working,
            Status::NeedsInput => self.needs_input,
            Status::Error => self.error,
            Status::Exited => self.exited,
        }
    }

    /// Applies one `[theme]` key. Unknown keys are left to the caller to report.
    pub fn set(&mut self, key: &str, color: Color) -> bool {
        match key {
            "border" => self.border = color,
            "text" => self.text = color,
            "dim" => self.dim = color,
            "idle" => self.idle = color,
            "working" => self.working = color,
            "needs_input" => self.needs_input = color,
            "error" => self.error = color,
            "exited" => self.exited = color,
            _ => return false,
        }
        true
    }
}

/// `#rrggbb`, or one of the terminal's own colour names.
pub fn parse_color(value: &str) -> Option<Color> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix('#') {
        if hex.len() != 6 {
            return None;
        }
        let channel = |at: usize| u8::from_str_radix(&hex[at..at + 2], 16).ok();
        return Some(Color::Rgb(channel(0)?, channel(2)?, channel(4)?));
    }
    match value.to_ascii_lowercase().replace('-', "_").as_str() {
        "black" => Some(Color::Black),
        "red" => Some(Color::Red),
        "green" => Some(Color::Green),
        "yellow" => Some(Color::Yellow),
        "blue" => Some(Color::Blue),
        "magenta" => Some(Color::Magenta),
        "cyan" => Some(Color::Cyan),
        "gray" | "grey" => Some(Color::Gray),
        "dark_gray" | "dark_grey" => Some(Color::DarkGray),
        "white" => Some(Color::White),
        "light_red" => Some(Color::LightRed),
        "light_green" => Some(Color::LightGreen),
        "light_yellow" => Some(Color::LightYellow),
        "light_blue" => Some(Color::LightBlue),
        "light_magenta" => Some(Color::LightMagenta),
        "light_cyan" => Some(Color::LightCyan),
        "reset" | "default" => Some(Color::Reset),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../tests/helpers/palette.rs"]
mod tests;
