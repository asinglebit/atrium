use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// xterm's modifier parameter: 1, plus shift 1, alt 2, ctrl 4.
fn modifier_param(mods: KeyModifiers) -> u8 {
    1 + u8::from(mods.contains(KeyModifiers::SHIFT)) + 2 * u8::from(mods.contains(KeyModifiers::ALT)) + 4 * u8::from(mods.contains(KeyModifiers::CONTROL))
}

/// Keys that end in a letter, like the arrows: ESC[A, or ESC[1;5A when modified.
fn csi_letter(letter: char, mods: KeyModifiers) -> Vec<u8> {
    match modifier_param(mods) {
        1 => format!("\x1b[{letter}").into_bytes(),
        m => format!("\x1b[1;{m}{letter}").into_bytes(),
    }
}

/// Keys addressed by number, like PageUp: ESC[5~, or ESC[5;5~ when modified.
fn csi_tilde(num: u8, mods: KeyModifiers) -> Vec<u8> {
    match modifier_param(mods) {
        1 => format!("\x1b[{num}~").into_bytes(),
        m => format!("\x1b[{num};{m}~").into_bytes(),
    }
}

/// F1-F4 are the odd ones out: SS3 rather than CSI while unmodified.
fn function_key(n: u8, mods: KeyModifiers) -> Vec<u8> {
    let m = modifier_param(mods);
    match n {
        1..=4 => {
            let letter = (b'P' + (n - 1)) as char;
            if m == 1 { format!("\x1bO{letter}").into_bytes() } else { format!("\x1b[1;{m}{letter}").into_bytes() }
        },
        _ => {
            // The numbering upstream of F5 is not contiguous; this is the table.
            let num = match n {
                5 => 15,
                6 => 17,
                7 => 18,
                8 => 19,
                9 => 20,
                10 => 21,
                11 => 23,
                12 => 24,
                _ => return Vec::new(),
            };
            csi_tilde(num, mods)
        },
    }
}

/// The control byte a ctrl+<char> chord sends, where one exists.
fn control_byte(c: char) -> Option<u8> {
    match c.to_ascii_lowercase() {
        c @ 'a'..='z' => Some(c as u8 - b'a' + 1),
        ' ' | '@' => Some(0x00),
        '[' => Some(0x1b),
        '\\' => Some(0x1c),
        ']' => Some(0x1d),
        '^' => Some(0x1e),
        '_' => Some(0x1f),
        '?' => Some(0x7f),
        _ => None,
    }
}

/// Turns a key press into the bytes a terminal would have sent for it.
pub fn encode(key: KeyEvent) -> Option<Vec<u8>> {
    let mods = key.modifiers;
    let alt = mods.contains(KeyModifiers::ALT);

    // Alt is a leading ESC on anything that is otherwise a plain byte.
    let with_alt = |mut bytes: Vec<u8>| {
        if alt {
            bytes.insert(0, 0x1b);
        }
        bytes
    };

    let bytes = match key.code {
        KeyCode::Char(c) => {
            if mods.contains(KeyModifiers::CONTROL) {
                match control_byte(c) {
                    Some(b) => with_alt(vec![b]),
                    None => with_alt(c.to_string().into_bytes()),
                }
            } else {
                with_alt(c.to_string().into_bytes())
            }
        },
        KeyCode::Enter => with_alt(vec![b'\r']),
        KeyCode::Tab => with_alt(vec![b'\t']),
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Backspace => with_alt(vec![0x7f]),
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => csi_letter('A', mods),
        KeyCode::Down => csi_letter('B', mods),
        KeyCode::Right => csi_letter('C', mods),
        KeyCode::Left => csi_letter('D', mods),
        KeyCode::Home => csi_letter('H', mods),
        KeyCode::End => csi_letter('F', mods),
        KeyCode::Insert => csi_tilde(2, mods),
        KeyCode::Delete => csi_tilde(3, mods),
        KeyCode::PageUp => csi_tilde(5, mods),
        KeyCode::PageDown => csi_tilde(6, mods),
        KeyCode::F(n) => function_key(n, mods),
        _ => return None,
    };

    if bytes.is_empty() { None } else { Some(bytes) }
}

/// Wraps pasted text so the agent can tell it apart from typing.
pub fn encode_paste(text: &str) -> Vec<u8> {
    let mut out = b"\x1b[200~".to_vec();
    out.extend_from_slice(text.as_bytes());
    out.extend_from_slice(b"\x1b[201~");
    out
}

#[cfg(test)]
#[path = "../../tests/app/input/keys.rs"]
mod tests;
