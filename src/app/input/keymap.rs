use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// A key plus the modifiers held with it.
///
/// Note there is no such thing as a bare-modifier chord: terminals report a
/// modifier only as part of some other key, so `ctrl` alone can never be bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chord {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

/// Terminals disagree about whether a shifted letter arrives uppercase or as a
/// SHIFT flag, so both sides are settled on lowercase-plus-flag before they are
/// compared. Modifiers beyond these three never reach a terminal application.
fn normalise(code: KeyCode, modifiers: KeyModifiers) -> (KeyCode, KeyModifiers) {
    let modifiers = modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
    match code {
        KeyCode::Char(c) if c.is_uppercase() => (KeyCode::Char(c.to_ascii_lowercase()), modifiers | KeyModifiers::SHIFT),
        _ => (code, modifiers),
    }
}

impl Chord {
    pub fn plain(code: KeyCode) -> Self {
        Self { code, modifiers: KeyModifiers::NONE }
    }

    pub fn matches(&self, event: &KeyEvent) -> bool {
        normalise(self.code, self.modifiers) == normalise(event.code, event.modifiers)
    }

    /// How this chord is written in the config file, so errors and `--help` can
    /// quote something you could paste back.
    pub fn label(&self) -> String {
        let mut out = String::new();
        for (flag, name) in [(KeyModifiers::CONTROL, "ctrl"), (KeyModifiers::ALT, "alt"), (KeyModifiers::SHIFT, "shift")] {
            if self.modifiers.contains(flag) {
                out.push_str(name);
                out.push('+');
            }
        }
        out.push_str(&key_name(self.code));
        out
    }
}

fn key_name(code: KeyCode) -> String {
    match code {
        KeyCode::Char(' ') => "space".to_owned(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::F(n) => format!("f{n}"),
        KeyCode::Esc => "esc".to_owned(),
        KeyCode::Tab => "tab".to_owned(),
        KeyCode::Enter => "enter".to_owned(),
        KeyCode::Backspace => "backspace".to_owned(),
        KeyCode::Up => "up".to_owned(),
        KeyCode::Down => "down".to_owned(),
        KeyCode::Left => "left".to_owned(),
        KeyCode::Right => "right".to_owned(),
        KeyCode::Home => "home".to_owned(),
        KeyCode::End => "end".to_owned(),
        KeyCode::Insert => "insert".to_owned(),
        KeyCode::Delete => "delete".to_owned(),
        other => format!("{other:?}").to_lowercase(),
    }
}

/// Which key does what. Everything but the leader is a chord pressed after it,
/// so these never collide with what the agent wants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keymap {
    pub leader: Chord,
    pub quit: Chord,
    pub new: Chord,
    pub dismiss: Chord,
    pub next: Chord,
    pub previous: Chord,
}

impl Default for Keymap {
    /// F12 is the leader because nothing else in this stack claims it -- not
    /// sway, ghostty, tmux, vim or readline -- so it can be taken without
    /// costing the agent a key it wanted.
    fn default() -> Self {
        Self {
            leader: Chord::plain(KeyCode::F(12)),
            quit: Chord::plain(KeyCode::Char('q')),
            new: Chord::plain(KeyCode::Char('n')),
            dismiss: Chord::plain(KeyCode::Char('x')),
            next: Chord::plain(KeyCode::Char('j')),
            previous: Chord::plain(KeyCode::Char('k')),
        }
    }
}

impl Keymap {
    /// Applies one `[keys]` entry. False when the name is not one of ours.
    pub fn set(&mut self, action: &str, chord: Chord) -> bool {
        match action {
            "leader" => self.leader = chord,
            "quit" => self.quit = chord,
            "new" => self.new = chord,
            "dismiss" => self.dismiss = chord,
            "next" => self.next = chord,
            "previous" => self.previous = chord,
            _ => return false,
        }
        true
    }
}

/// A chord by name: `q`, `f12`, `ctrl+g`, `ctrl+shift+p`, `alt+enter`.
///
/// `+` cannot itself be bound, which keeps the separator unambiguous.
pub fn parse_chord(value: &str) -> Option<Chord> {
    let value = value.trim();
    let mut modifiers = KeyModifiers::NONE;

    let mut parts = value.split('+').peekable();
    let mut code = None;
    while let Some(part) = parts.next() {
        if part.is_empty() {
            return None;
        }
        if parts.peek().is_none() {
            code = parse_key(part);
            break;
        }
        modifiers |= match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" => KeyModifiers::CONTROL,
            "alt" | "meta" | "opt" | "option" => KeyModifiers::ALT,
            "shift" => KeyModifiers::SHIFT,
            _ => return None,
        };
    }

    Some(Chord { code: code?, modifiers })
}

/// A single key by name, with no modifiers: `q`, `f12`, `esc`, `tab`, `space`.
pub fn parse_key(value: &str) -> Option<KeyCode> {
    let mut chars = value.chars();
    // A lone character is itself, so a literal "f" is not read as a function key.
    if let (Some(c), None) = (chars.next(), chars.next()) {
        return Some(KeyCode::Char(c));
    }

    let lower = value.to_ascii_lowercase();
    if let Some(number) = lower.strip_prefix('f')
        && let Ok(n) = number.parse::<u8>()
        && (1..=12).contains(&n)
    {
        return Some(KeyCode::F(n));
    }

    match lower.as_str() {
        "esc" | "escape" => Some(KeyCode::Esc),
        "tab" => Some(KeyCode::Tab),
        "space" => Some(KeyCode::Char(' ')),
        "enter" | "return" => Some(KeyCode::Enter),
        "backspace" => Some(KeyCode::Backspace),
        "up" => Some(KeyCode::Up),
        "down" => Some(KeyCode::Down),
        "left" => Some(KeyCode::Left),
        "right" => Some(KeyCode::Right),
        "home" => Some(KeyCode::Home),
        "end" => Some(KeyCode::End),
        "insert" => Some(KeyCode::Insert),
        "delete" => Some(KeyCode::Delete),
        _ => None,
    }
}

#[cfg(test)]
#[path = "../../tests/app/input/keymap.rs"]
mod tests;
