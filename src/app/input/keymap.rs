use crossterm::event::KeyCode;

/// Which key does what. Everything but the leader is a chord pressed after it,
/// so these never collide with what the agent wants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keymap {
    pub leader: KeyCode,
    pub quit: KeyCode,
    pub new: KeyCode,
    pub dismiss: KeyCode,
    pub next: KeyCode,
    pub previous: KeyCode,
}

impl Default for Keymap {
    /// F12 is the leader because nothing else in this stack claims it -- not
    /// sway, ghostty, tmux, vim or readline -- so it can be taken without
    /// costing the agent a key it wanted.
    fn default() -> Self {
        Self { leader: KeyCode::F(12), quit: KeyCode::Char('q'), new: KeyCode::Char('n'), dismiss: KeyCode::Char('x'), next: KeyCode::Char('j'), previous: KeyCode::Char('k') }
    }
}

impl Keymap {
    /// Applies one `[keys]` entry. False when the name is not one of ours.
    pub fn set(&mut self, action: &str, key: KeyCode) -> bool {
        match action {
            "leader" => self.leader = key,
            "quit" => self.quit = key,
            "new" => self.new = key,
            "dismiss" => self.dismiss = key,
            "next" => self.next = key,
            "previous" => self.previous = key,
            _ => return false,
        }
        true
    }
}

/// A single key by name: `q`, `f12`, `esc`, `tab`, `space`, `enter`.
pub fn parse_key(value: &str) -> Option<KeyCode> {
    let value = value.trim();
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
