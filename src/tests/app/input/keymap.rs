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
fn atrium_takes_one_chord_and_not_a_key_more() {
    assert_eq!(Keymap::default().claimed().len(), 1, "everything but the prefix belongs to the agent");
}

#[test]
fn the_keys_behind_the_prefix_cost_the_agent_nothing() {
    // They mean something only after the prefix, so a bare letter here is not
    // a letter taken away from anyone. shift is allowed -- a shifted key is
    // still one you type -- but ctrl and alt are what an agent wants back.
    for (name, chord) in Keymap::default().actions() {
        assert!(!chord.modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT), "{name} is {}, which would be claimed outright", chord.label());
    }
}

#[test]
fn the_prefix_is_the_one_readline_key_given_up() {
    // ctrl+[ is Escape, and the rest are signals or readline editing that a
    // shell or an agent would miss immediately. ctrl+space costs readline's
    // set-mark and nothing else, which is the cheapest key there was to take.
    const RESERVED: [char; 11] = ['c', 'd', 'z', 'v', 'l', 'r', 'u', 'w', 'e', 'k', '['];

    for chord in Keymap::default().claimed() {
        if let KeyCode::Char(c) = chord.code {
            assert!(!RESERVED.contains(&c.to_ascii_lowercase()), "ctrl+{c} belongs to the agent");
        }
    }
}

#[test]
fn no_default_is_a_chord_a_terminal_cannot_deliver() {
    // The legacy encoding has a byte for ctrl plus a letter, one for ctrl+space
    // -- NUL, which crossterm reports as itself -- and for very little else:
    // ctrl+] arrives as ctrl+5, and ctrl+1 as a bare 1. A default bound to
    // either could never fire, which is exactly what ctrl+] did here.
    for chord in Keymap::default().claimed() {
        if chord.modifiers.contains(KeyModifiers::CONTROL)
            && let KeyCode::Char(c) = chord.code
        {
            assert!(c.is_ascii_alphabetic() || c == ' ', "ctrl+{c} never reaches an application as itself");
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
    for c in ['c', 'd', 'z', 'v', 'l'] {
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

#[test]
fn the_prefix_is_the_one_nothing_above_atrium_wants() {
    // tmux's prefix here is C-a, so ctrl+space reaches atrium on the first
    // press rather than having to be sent through with a second one.
    assert_eq!(Keymap::default().action.label(), "ctrl+space");
}

#[test]
fn the_defaults_are_the_ones_asked_for() {
    let keymap = Keymap::default();
    for (name, expected) in [("sidebar", "shift+1"), ("settings", "?"), ("new", "n"), ("dismiss", "x"), ("quit", "q"), ("next", "j"), ("previous", "k"), ("goto", "space")] {
        let chord = keymap.actions().into_iter().find(|(action, _)| *action == name).expect("a binding").1;
        assert_eq!(chord.label(), expected, "{name}");
    }
}

#[test]
fn a_key_after_the_prefix_names_its_action() {
    let keymap = Keymap::default();
    assert_eq!(keymap.action_for(&press(KeyCode::Char('!'), KeyModifiers::NONE)), Some("sidebar"));
    assert_eq!(keymap.action_for(&press(KeyCode::Char(' '), KeyModifiers::NONE)), Some("goto"));
    assert_eq!(keymap.action_for(&press(KeyCode::Char('?'), KeyModifiers::NONE)), Some("settings"));
    assert_eq!(keymap.action_for(&press(KeyCode::Char('n'), KeyModifiers::NONE)), Some("new"));
    // A bare digit is an agent rather than an action; the caller asks for that
    // next, which is what keeps a digit someone has bound to an action theirs.
    assert_eq!(keymap.action_for(&press(KeyCode::Char('1'), KeyModifiers::NONE)), None);
}

#[test]
fn a_key_that_means_nothing_names_no_action() {
    // The caller cancels on None rather than passing it on, so half a mistyped
    // gesture cannot land in a conversation.
    assert!(Keymap::default().action_for(&press(KeyCode::Char('z'), KeyModifiers::NONE)).is_none());
    assert_eq!(Keymap::default().action_for(&press(KeyCode::Esc, KeyModifiers::NONE)), Some("leave"), "esc is the way out of the mode");
}

#[test]
fn the_old_chords_now_reach_the_agent() {
    // ctrl+t, ctrl+n and ctrl+p used to be atrium's. An agent gets them back,
    // which is the whole point of the prefix.
    let keymap = Keymap::default();
    for c in ['t', 'n', 'p', 'x', 'g', 'o', 's', 'q'] {
        let event = press(KeyCode::Char(c), KeyModifiers::CONTROL);
        assert!(!keymap.action.matches(&event), "ctrl+{c} should reach the agent");
    }
}

#[test]
fn the_gesture_reads_as_both_keys() {
    let keymap = Keymap::default();
    assert_eq!(keymap.gesture(keymap.sidebar), "ctrl+space shift+1");
    assert_eq!(keymap.gesture(keymap.goto), "ctrl+space space");
    assert_eq!(keymap.gesture(keymap.settings), "ctrl+space ?");
}

#[test]
fn a_shifted_digit_matches_the_character_the_terminal_sends() {
    // There is no SHIFT flag to read in the legacy encoding: shift+1 arrives
    // as `!` and nothing else, so that is what the chord has to match.
    let bound = chord("shift+1");
    assert!(bound.matches(&press(KeyCode::Char('!'), KeyModifiers::NONE)));
    assert!(bound.matches(&press(KeyCode::Char('1'), KeyModifiers::SHIFT)));
    assert!(!bound.matches(&press(KeyCode::Char('1'), KeyModifiers::NONE)), "a bare digit names an agent, not the sidebar");
}

#[test]
fn a_shifted_digit_can_be_written_either_way() {
    let typed = press(KeyCode::Char('!'), KeyModifiers::NONE);
    assert!(chord("!").matches(&typed), "the character a terminal sends");
    assert!(chord("shift+1").matches(&typed), "the key you actually press");
}

#[test]
fn a_digit_names_the_agent_on_that_row() {
    assert_eq!(agent_for(&press(KeyCode::Char('1'), KeyModifiers::NONE)), Some(0));
    assert_eq!(agent_for(&press(KeyCode::Char('9'), KeyModifiers::NONE)), Some(8));
    assert_eq!(agent_for(&press(KeyCode::Char('0'), KeyModifiers::NONE)), Some(9), "zero is the tenth row, not the first");
}

#[test]
fn a_digit_that_is_not_bare_names_no_agent() {
    // shift+1 is the sidebar, and ctrl+1 is not a key a terminal can send.
    assert_eq!(agent_for(&press(KeyCode::Char('!'), KeyModifiers::NONE)), None);
    assert_eq!(agent_for(&press(KeyCode::Char('1'), KeyModifiers::CONTROL)), None);
    assert_eq!(agent_for(&press(KeyCode::Char('n'), KeyModifiers::NONE)), None);
}

#[test]
fn ctrl_j_and_ctrl_k_walk_a_list() {
    assert_eq!(as_arrow(press(KeyCode::Char('j'), KeyModifiers::CONTROL)).code, KeyCode::Down);
    assert_eq!(as_arrow(press(KeyCode::Char('k'), KeyModifiers::CONTROL)).code, KeyCode::Up);
}

#[test]
fn everything_else_reaches_the_list_as_itself() {
    // A bare letter is text being typed in the boxes that take text, so only
    // the ctrl pair is turned into anything.
    for key in [press(KeyCode::Char('j'), KeyModifiers::NONE), press(KeyCode::Char('k'), KeyModifiers::NONE), press(KeyCode::Char('x'), KeyModifiers::CONTROL)] {
        assert_eq!(as_arrow(key).code, key.code);
    }
}
