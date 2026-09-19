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

use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> MouseEvent {
    MouseEvent { kind, column, row, modifiers: KeyModifiers::NONE }
}

#[test]
fn a_click_becomes_an_sgr_report() {
    let event = mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert_eq!(encode_mouse(&event, (0, 0)).expect("encodes"), b"\x1b[<0;1;1M");
}

#[test]
fn a_release_ends_in_a_lowercase_m() {
    let event = mouse(MouseEventKind::Up(MouseButton::Left), 0, 0);
    assert_eq!(encode_mouse(&event, (0, 0)).expect("encodes"), b"\x1b[<0;1;1m");
}

#[test]
fn coordinates_are_relative_to_the_agents_own_corner() {
    // The agent starts at column 28: a click there is its column 1, not 29.
    let event = mouse(MouseEventKind::Down(MouseButton::Left), 30, 5);
    assert_eq!(encode_mouse(&event, (28, 3)).expect("encodes"), b"\x1b[<0;3;3M");
}

#[test]
fn a_click_outside_the_agent_does_not_encode() {
    let event = mouse(MouseEventKind::Down(MouseButton::Left), 2, 5);
    assert!(encode_mouse(&event, (28, 3)).is_none(), "a click left of the stage is not the agent's");
}

#[test]
fn the_buttons_are_numbered_the_way_terminals_number_them() {
    for (button, code) in [(MouseButton::Left, 0), (MouseButton::Middle, 1), (MouseButton::Right, 2)] {
        let event = mouse(MouseEventKind::Down(button), 0, 0);
        assert_eq!(encode_mouse(&event, (0, 0)).expect("encodes"), format!("\x1b[<{code};1;1M").into_bytes());
    }
}

#[test]
fn the_wheel_reports_its_own_codes() {
    assert_eq!(encode_mouse(&mouse(MouseEventKind::ScrollUp, 0, 0), (0, 0)).expect("encodes"), b"\x1b[<64;1;1M");
    assert_eq!(encode_mouse(&mouse(MouseEventKind::ScrollDown, 0, 0), (0, 0)).expect("encodes"), b"\x1b[<65;1;1M");
}

#[test]
fn a_drag_sets_the_motion_bit() {
    let event = mouse(MouseEventKind::Drag(MouseButton::Left), 0, 0);
    assert_eq!(encode_mouse(&event, (0, 0)).expect("encodes"), b"\x1b[<32;1;1M");
}

#[test]
fn modifiers_ride_along_with_the_button() {
    let mut event = mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    event.modifiers = KeyModifiers::CONTROL;
    assert_eq!(encode_mouse(&event, (0, 0)).expect("encodes"), b"\x1b[<16;1;1M");
}
