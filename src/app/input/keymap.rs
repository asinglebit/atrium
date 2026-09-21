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
///
/// A shifted digit is settled the same way. There is no SHIFT flag to read: the
/// terminal sends `!`, so `!` and `shift+1` are made into the one chord, and
/// either can be written in the config file.
fn normalise(code: KeyCode, modifiers: KeyModifiers) -> (KeyCode, KeyModifiers) {
    let modifiers = modifiers & (KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT);
    match code {
        KeyCode::Char(c) if c.is_uppercase() => (KeyCode::Char(c.to_ascii_lowercase()), modifiers | KeyModifiers::SHIFT),
        KeyCode::Char(c) => match unshift_digit(c) {
            Some(digit) => (KeyCode::Char(digit), modifiers | KeyModifiers::SHIFT),
            None => (code, modifiers),
        },
        _ => (code, modifiers),
    }
}

/// The digit a shifted-digit character is typed on. US layout, which is the one
/// the defaults are written for; on another, the key still works, it is just
/// named after the wrong digit.
fn unshift_digit(c: char) -> Option<char> {
    let index = "!@#$%^&*()".find(c)?;
    "1234567890".chars().nth(index)
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

/// Which key does what, once `action` has been pressed.
///
/// Nothing here fires on its own. Every keystroke reaches the agent, and the
/// only chord atrium takes is `action` -- so an agent keeps `ctrl+t`,
/// `ctrl+n`, `ctrl+p` and the rest, which it never did before. This is
/// guitar's action mode, for the same reason: the keys worth having are the
/// ones the thing underneath is not already using.
///
/// `ctrl+space` is nobody else's here: tmux's prefix is `C-a`, so it passes
/// straight through rather than having to be pressed twice.
///
/// The digits name the rows, so `1` is the first agent and `0` the tenth. That
/// is what moves `1` off the sidebar and onto `shift+1`. The rest are guitar's
/// where guitar has one: `?` is settings, `x` drops a thing, `q` exits, and
/// `j`/`k` walk a list.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Keymap {
    /// The one chord atrium takes from the agent. Everything else follows it.
    pub action: Chord,
    /// The way back out. Only ever read while atrium has the keyboard, so it
    /// costs the agent nothing to have it be a key the agent also uses.
    pub leave: Chord,
    pub quit: Chord,
    pub goto: Chord,
    pub settings: Chord,
    pub sidebar: Chord,
    pub new: Chord,
    pub dismiss: Chord,
    pub next: Chord,
    pub previous: Chord,
}

fn ctrl(c: char) -> Chord {
    Chord { code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL }
}

fn key(c: char) -> Chord {
    Chord::plain(KeyCode::Char(c))
}

fn shift(c: char) -> Chord {
    Chord { code: KeyCode::Char(c), modifiers: KeyModifiers::SHIFT }
}

impl Default for Keymap {
    fn default() -> Self {
        Self {
            action: ctrl(' '),
            leave: Chord::plain(KeyCode::Esc),
            quit: key('q'),
            goto: key(' '),
            settings: key('?'),
            sidebar: shift('1'),
            new: key('n'),
            dismiss: key('x'),
            next: key('j'),
            previous: key('k'),
        }
    }
}

impl Keymap {
    /// Applies one `[keys]` entry. False when the name is not one of ours.
    pub fn set(&mut self, action: &str, chord: Chord) -> bool {
        match action {
            "action" => self.action = chord,
            "leave" => self.leave = chord,
            "quit" => self.quit = chord,
            "goto" => self.goto = chord,
            "settings" => self.settings = chord,
            "sidebar" => self.sidebar = chord,
            "new" => self.new = chord,
            "dismiss" => self.dismiss = chord,
            "next" => self.next = chord,
            "previous" => self.previous = chord,
            _ => return false,
        }
        true
    }

    /// Every action with the name the config file uses, for the settings list.
    pub fn actions(&self) -> [(&'static str, Chord); 9] {
        [
            ("leave", self.leave),
            ("new", self.new),
            ("goto", self.goto),
            ("next", self.next),
            ("previous", self.previous),
            ("dismiss", self.dismiss),
            ("sidebar", self.sidebar),
            ("settings", self.settings),
            ("quit", self.quit),
        ]
    }

    /// The one chord atrium claims. Everything else the agent still sees --
    /// which is the whole point of putting the actions behind a prefix.
    pub fn claimed(&self) -> [Chord; 1] {
        [self.action]
    }

    /// How a binding is written out for a reader: the prefix, then the key.
    pub fn gesture(&self, chord: Chord) -> String {
        format!("{} {}", self.action.label(), chord.label())
    }

    /// The same, for a list that names its actions. The way out is the one key
    /// not reached through the prefix -- it is pressed on its own, once atrium
    /// already has the keyboard -- so quoting it with the prefix would be
    /// telling someone to press a key that does something else.
    pub fn gesture_for(&self, action: &str, chord: Chord) -> String {
        if action == "leave" { chord.label() } else { self.gesture(chord) }
    }

    /// The action this key runs, once the prefix has been pressed.
    pub fn action_for(&self, key: &KeyEvent) -> Option<&'static str> {
        self.actions().into_iter().find(|(_, chord)| chord.matches(key)).map(|(name, _)| name)
    }
}

/// ctrl+j and ctrl+k as the arrows they stand for, so a list can be walked
/// without leaving the home row and without the letter being taken for text.
/// Only ever called where atrium is taking the keys rather than the agent.
pub fn as_arrow(key: KeyEvent) -> KeyEvent {
    if !key.modifiers.contains(KeyModifiers::CONTROL) {
        return key;
    }
    match key.code {
        KeyCode::Char('j') => KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        KeyCode::Char('k') => KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
        _ => key,
    }
}

/// The row a digit names, counting from one, with `0` for the tenth. Ten keys
/// is as far as one keystroke goes, and it is as far as the rows are numbered.
pub fn row_for_digit(digit: char) -> Option<usize> {
    match digit.to_digit(10)? as usize {
        0 => Some(9),
        n => Some(n - 1),
    }
}

/// The agent a key names, once the prefix has been pressed. Not rebindable:
/// these are the numbers written down the sidebar, and they are what they are.
pub fn agent_for(key: &KeyEvent) -> Option<usize> {
    let (code, modifiers) = normalise(key.code, key.modifiers);
    match code {
        KeyCode::Char(c) if modifiers.is_empty() => row_for_digit(c),
        _ => None,
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
