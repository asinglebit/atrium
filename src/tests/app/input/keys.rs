use super::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

fn bytes(code: KeyCode, mods: KeyModifiers) -> Vec<u8> {
    encode(KeyEvent::new(code, mods)).expect("key should encode")
}

#[test]
fn plain_characters_pass_through() {
    assert_eq!(bytes(KeyCode::Char('a'), KeyModifiers::NONE), b"a");
    assert_eq!(bytes(KeyCode::Char('Z'), KeyModifiers::SHIFT), b"Z");
}

#[test]
fn multibyte_characters_survive() {
    assert_eq!(bytes(KeyCode::Char('ж'), KeyModifiers::NONE), "ж".as_bytes());
}

#[test]
fn control_chords_become_control_bytes() {
    assert_eq!(bytes(KeyCode::Char('c'), KeyModifiers::CONTROL), [0x03]);
    assert_eq!(bytes(KeyCode::Char('a'), KeyModifiers::CONTROL), [0x01]);
    assert_eq!(bytes(KeyCode::Char('['), KeyModifiers::CONTROL), [0x1b]);
    assert_eq!(bytes(KeyCode::Char(' '), KeyModifiers::CONTROL), [0x00]);
}

#[test]
fn control_is_case_insensitive() {
    assert_eq!(bytes(KeyCode::Char('C'), KeyModifiers::CONTROL), [0x03]);
}

#[test]
fn alt_prefixes_an_escape() {
    assert_eq!(bytes(KeyCode::Char('b'), KeyModifiers::ALT), b"\x1bb");
}

#[test]
fn named_keys_use_their_control_bytes() {
    assert_eq!(bytes(KeyCode::Enter, KeyModifiers::NONE), b"\r");
    assert_eq!(bytes(KeyCode::Tab, KeyModifiers::NONE), b"\t");
    assert_eq!(bytes(KeyCode::Backspace, KeyModifiers::NONE), [0x7f]);
    assert_eq!(bytes(KeyCode::Esc, KeyModifiers::NONE), [0x1b]);
    assert_eq!(bytes(KeyCode::BackTab, KeyModifiers::NONE), b"\x1b[Z");
}

#[test]
fn arrows_are_csi_sequences() {
    assert_eq!(bytes(KeyCode::Up, KeyModifiers::NONE), b"\x1b[A");
    assert_eq!(bytes(KeyCode::Down, KeyModifiers::NONE), b"\x1b[B");
    assert_eq!(bytes(KeyCode::Right, KeyModifiers::NONE), b"\x1b[C");
    assert_eq!(bytes(KeyCode::Left, KeyModifiers::NONE), b"\x1b[D");
}

#[test]
fn modified_arrows_carry_the_modifier_parameter() {
    assert_eq!(bytes(KeyCode::Up, KeyModifiers::SHIFT), b"\x1b[1;2A");
    assert_eq!(bytes(KeyCode::Up, KeyModifiers::ALT), b"\x1b[1;3A");
    assert_eq!(bytes(KeyCode::Up, KeyModifiers::CONTROL), b"\x1b[1;5A");
    assert_eq!(bytes(KeyCode::Left, KeyModifiers::CONTROL | KeyModifiers::SHIFT), b"\x1b[1;6D");
}

#[test]
fn navigation_keys_are_numbered() {
    assert_eq!(bytes(KeyCode::Insert, KeyModifiers::NONE), b"\x1b[2~");
    assert_eq!(bytes(KeyCode::Delete, KeyModifiers::NONE), b"\x1b[3~");
    assert_eq!(bytes(KeyCode::PageUp, KeyModifiers::NONE), b"\x1b[5~");
    assert_eq!(bytes(KeyCode::PageDown, KeyModifiers::NONE), b"\x1b[6~");
    assert_eq!(bytes(KeyCode::Home, KeyModifiers::NONE), b"\x1b[H");
    assert_eq!(bytes(KeyCode::End, KeyModifiers::NONE), b"\x1b[F");
}

#[test]
fn f1_to_f4_use_ss3_but_the_rest_use_csi() {
    assert_eq!(bytes(KeyCode::F(1), KeyModifiers::NONE), b"\x1bOP");
    assert_eq!(bytes(KeyCode::F(4), KeyModifiers::NONE), b"\x1bOS");
    assert_eq!(bytes(KeyCode::F(5), KeyModifiers::NONE), b"\x1b[15~");
    assert_eq!(bytes(KeyCode::F(12), KeyModifiers::NONE), b"\x1b[24~");
}

#[test]
fn modified_f1_switches_to_csi() {
    assert_eq!(bytes(KeyCode::F(1), KeyModifiers::CONTROL), b"\x1b[1;5P");
}

#[test]
fn unknown_keys_are_dropped_rather_than_guessed() {
    assert!(encode(KeyEvent::new(KeyCode::Null, KeyModifiers::NONE)).is_none());
}

#[test]
fn paste_is_bracketed() {
    assert_eq!(encode_paste("hi"), b"\x1b[200~hi\x1b[201~");
}
