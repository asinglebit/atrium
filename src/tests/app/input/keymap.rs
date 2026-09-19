use super::*;

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
    assert_eq!(parse_key("f99"), None);
}

#[test]
fn named_keys_parse() {
    assert_eq!(parse_key("esc"), Some(KeyCode::Esc));
    assert_eq!(parse_key("escape"), Some(KeyCode::Esc));
    assert_eq!(parse_key("tab"), Some(KeyCode::Tab));
    assert_eq!(parse_key("space"), Some(KeyCode::Char(' ')));
    assert_eq!(parse_key("enter"), Some(KeyCode::Enter));
    assert_eq!(parse_key("down"), Some(KeyCode::Down));
}

#[test]
fn nonsense_is_not_a_key() {
    assert_eq!(parse_key("ctrl+shift+meta"), None);
    assert_eq!(parse_key(""), None);
}

#[test]
fn the_default_leader_is_f12() {
    assert_eq!(Keymap::default().leader, KeyCode::F(12));
}

#[test]
fn every_action_is_distinct_by_default() {
    let map = Keymap::default();
    let bound = [map.leader, map.quit, map.new, map.dismiss, map.next, map.previous];
    for (index, key) in bound.iter().enumerate() {
        assert!(!bound[index + 1..].contains(key), "{key:?} is bound to two actions");
    }
}

#[test]
fn setting_a_known_action_takes_and_an_unknown_one_does_not() {
    let mut map = Keymap::default();
    assert!(map.set("quit", KeyCode::Esc));
    assert_eq!(map.quit, KeyCode::Esc);
    assert!(!map.set("qit", KeyCode::Esc));
}
