use super::*;

use crossterm::event::{KeyCode, KeyModifiers};

use crate::app::input::keymap::Chord;

fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

fn action() -> KeyEvent {
    press(KeyCode::Char(' '), KeyModifiers::CONTROL)
}

fn esc() -> KeyEvent {
    press(KeyCode::Esc, KeyModifiers::NONE)
}

fn letter(c: char) -> KeyEvent {
    press(KeyCode::Char(c), KeyModifiers::NONE)
}

/// The mode as it stands on the stage, where it is sticky.
fn on_stage() -> (Mode, Keymap) {
    (Mode::default(), Keymap::default())
}

#[test]
fn the_agent_has_the_keyboard_until_it_is_asked_for() {
    let (mut mode, keymap) = on_stage();

    assert!(!mode.is_on());
    assert_eq!(mode.step(&letter('j'), &keymap, true), Step::Pass, "a letter is the agent's");
    assert_eq!(mode.step(&esc(), &keymap, true), Step::Pass, "and so is esc, which agents lean on");
    assert!(!mode.is_on());
}

#[test]
fn the_chord_takes_the_keyboard_and_gives_it_back() {
    let (mut mode, keymap) = on_stage();

    assert_eq!(mode.step(&action(), &keymap, true), Step::Entered);
    assert!(mode.is_on());
    assert_eq!(mode.step(&action(), &keymap, true), Step::Left, "the way in is also the way out");
    assert!(!mode.is_on());
}

#[test]
fn esc_gives_the_keyboard_back() {
    let (mut mode, keymap) = on_stage();
    mode.step(&action(), &keymap, true);

    assert_eq!(mode.step(&esc(), &keymap, true), Step::Left);
    assert!(!mode.is_on());
}

#[test]
fn an_action_does_not_end_it() {
    // The whole point of the mode: a run of keys on one press of the chord.
    let (mut mode, keymap) = on_stage();
    mode.step(&action(), &keymap, true);

    for key in ['j', 'j', 'x', 'j'] {
        assert_eq!(mode.step(&letter(key), &keymap, true), Step::Act, "{key} should still be atrium's");
        assert!(mode.is_on(), "the mode has to survive {key}");
    }
}

#[test]
fn a_key_that_means_nothing_is_still_atriums() {
    // It is swallowed by the caller rather than passed on: half a mistyped
    // gesture landing in a conversation is worse than nothing happening.
    let (mut mode, keymap) = on_stage();
    mode.step(&action(), &keymap, true);

    assert_eq!(mode.step(&letter('z'), &keymap, true), Step::Act);
    assert!(mode.is_on(), "a mistyped key does not throw you out either");
}

#[test]
fn on_the_splash_the_chord_reaches_one_action_and_no_more() {
    // atrium already has the keyboard there, so there is nothing to jump out of
    // and a mode would only trap someone whose next key the splash wanted.
    let (mut mode, keymap) = on_stage();

    assert_eq!(mode.step(&action(), &keymap, false), Step::Entered);
    assert_eq!(mode.step(&letter('n'), &keymap, false), Step::Act, "the gesture still reaches the picker");
    assert!(!mode.is_on(), "and ends with it");
    assert_eq!(mode.step(&letter('j'), &keymap, false), Step::Pass, "so the splash list gets its own keys back");
}

#[test]
fn releasing_it_hands_the_keyboard_back() {
    let (mut mode, keymap) = on_stage();
    mode.step(&action(), &keymap, true);

    mode.release();

    assert!(!mode.is_on());
    assert_eq!(mode.step(&letter('j'), &keymap, true), Step::Pass, "a surface opening puts the keys back where they were");
}

#[test]
fn releasing_what_is_already_released_is_nothing() {
    let (mut mode, _) = on_stage();

    mode.release();

    assert!(!mode.is_on());
}

#[test]
fn a_rebound_way_out_is_the_one_that_works() {
    let keymap = Keymap { leave: Chord::plain(KeyCode::Char('z')), ..Default::default() };
    let mut mode = Mode::default();
    mode.step(&action(), &keymap, true);

    assert_eq!(mode.step(&esc(), &keymap, true), Step::Act, "esc is just a key once it is not the way out");
    assert_eq!(mode.step(&letter('z'), &keymap, true), Step::Left);
}
