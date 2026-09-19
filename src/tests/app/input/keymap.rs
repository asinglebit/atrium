use super::*;

fn chord(value: &str) -> Chord {
    parse_chord(value).unwrap_or_else(|| panic!("{value} should parse"))
}

fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent::new(code, modifiers)
}

#[test]
fn a_single_character_is_itself() {
    assert_eq!(parse_key("q"), Some(KeyCode::Char('q')));
    assert_eq!(parse_key("1"), Some(KeyCode::Char('1')));
}

#[test]
fn a_lone_f_is_a_letter_not_a_function_key() {
    assert_eq!(parse_key("f"), Some(KeyCode::Char('f')));
}

#[test]
fn function_keys_parse_across_the_range() {
    assert_eq!(parse_key("f1"), Some(KeyCode::F(1)));
    assert_eq!(parse_key("F12"), Some(KeyCode::F(12)));
}

#[test]
fn function_keys_outside_the_range_are_rejected() {
    assert_eq!(parse_key("f0"), None);
    assert_eq!(parse_key("f13"), None);
}

#[test]
fn named_keys_parse() {
    assert_eq!(parse_key("esc"), Some(KeyCode::Esc));
    assert_eq!(parse_key("space"), Some(KeyCode::Char(' ')));
    assert_eq!(parse_key("enter"), Some(KeyCode::Enter));
}

#[test]
fn a_plain_key_carries_no_modifiers() {
    assert_eq!(chord("q"), Chord::plain(KeyCode::Char('q')));
}

#[test]
fn modifiers_parse_in_front_of_the_key() {
    assert_eq!(chord("ctrl+g"), Chord { code: KeyCode::Char('g'), modifiers: KeyModifiers::CONTROL });
    assert_eq!(chord("alt+enter"), Chord { code: KeyCode::Enter, modifiers: KeyModifiers::ALT });
    assert_eq!(chord("ctrl+alt+f4").modifiers, KeyModifiers::CONTROL | KeyModifiers::ALT);
}

#[test]
fn modifier_names_have_the_obvious_synonyms() {
    assert_eq!(chord("control+g"), chord("ctrl+g"));
    assert_eq!(chord("meta+g"), chord("alt+g"));
}

#[test]
fn a_bare_modifier_is_not_a_chord() {
    // Terminals never report a modifier on its own, so binding one could only
    // ever be a key that does nothing.
    assert_eq!(parse_chord("ctrl"), None);
    assert_eq!(parse_chord("ctrl+"), None);
    assert_eq!(parse_chord("+g"), None);
    assert_eq!(parse_chord(""), None);
}

#[test]
fn an_unknown_modifier_is_rejected_rather_than_ignored() {
    assert_eq!(parse_chord("hyper+g"), None);
}

#[test]
fn a_chord_matches_the_key_event_it_describes() {
    assert!(chord("ctrl+g").matches(&press(KeyCode::Char('g'), KeyModifiers::CONTROL)));
    assert!(!chord("ctrl+g").matches(&press(KeyCode::Char('g'), KeyModifiers::NONE)));
    assert!(!chord("g").matches(&press(KeyCode::Char('g'), KeyModifiers::CONTROL)));
}

#[test]
fn a_shifted_letter_matches_whichever_way_the_terminal_reports_it() {
    let bound = chord("shift+a");
    assert!(bound.matches(&press(KeyCode::Char('A'), KeyModifiers::SHIFT)));
    assert!(bound.matches(&press(KeyCode::Char('a'), KeyModifiers::SHIFT)));
    assert!(bound.matches(&press(KeyCode::Char('A'), KeyModifiers::NONE)), "an uppercase char implies shift");
}

#[test]
fn modifiers_the_terminal_adds_on_its_own_are_ignored() {
    // crossterm can report NONE as an empty set or with unrelated bits; only
    // ctrl, alt and shift decide a match.
    assert!(chord("ctrl+g").matches(&press(KeyCode::Char('g'), KeyModifiers::CONTROL | KeyModifiers::SUPER)));
}

#[test]
fn a_chord_prints_the_way_it_is_written() {
    assert_eq!(chord("ctrl+g").label(), "ctrl+g");
    assert_eq!(chord("f12").label(), "f12");
    assert_eq!(chord("ctrl+alt+enter").label(), "ctrl+alt+enter");
    assert_eq!(chord("space").label(), "space");
}

#[test]
fn a_label_can_be_parsed_back() {
    for value in ["q", "f12", "ctrl+g", "alt+enter", "ctrl+alt+shift+x"] {
        assert_eq!(chord(&chord(value).label()), chord(value), "{value} did not round-trip");
    }
}

#[test]
fn every_action_fires_on_a_ctrl_chord() {
    for chord in Keymap::default().claimed() {
        assert!(chord.modifiers.contains(KeyModifiers::CONTROL), "{} is not a ctrl chord", chord.label());
    }
}

#[test]
fn no_default_takes_a_key_the_agent_needs() {
    // ctrl+[ is Escape, and the rest are signals or readline editing that a
    // shell or an agent would miss immediately.
    const RESERVED: [char; 13] = ['c', 'd', 'z', 'v', 'x', 'l', 'r', 'u', 'w', 'a', 'e', 'k', '['];

    for chord in Keymap::default().claimed() {
        if let KeyCode::Char(c) = chord.code {
            assert!(!RESERVED.contains(&c.to_ascii_lowercase()), "ctrl+{c} belongs to the agent");
        }
    }
}

#[test]
fn every_action_is_distinct_by_default() {
    let bound = Keymap::default().claimed();
    for (index, key) in bound.iter().enumerate() {
        assert!(!bound[index + 1..].contains(key), "{key:?} is bound to two actions");
    }
}

#[test]
fn an_unclaimed_chord_is_not_mistaken_for_an_action() {
    let claimed = Keymap::default().claimed();
    for c in ['c', 'd', 'z', 'v', 'x'] {
        let pressed = press(KeyCode::Char(c), KeyModifiers::CONTROL);
        assert!(!claimed.iter().any(|chord| chord.matches(&pressed)), "ctrl+{c} should reach the agent");
    }
}

#[test]
fn a_plain_letter_is_never_an_action() {
    let claimed = Keymap::default().claimed();
    for c in ['q', 't', 'n', 'p'] {
        let pressed = press(KeyCode::Char(c), KeyModifiers::NONE);
        assert!(!claimed.iter().any(|chord| chord.matches(&pressed)), "{c} without ctrl should reach the agent");
    }
}

#[test]
fn setting_a_known_action_takes_and_an_unknown_one_does_not() {
    let mut map = Keymap::default();
    assert!(map.set("quit", chord("ctrl+j")));
    assert_eq!(map.quit, chord("ctrl+j"));
    assert!(!map.set("qit", chord("x")));
}
