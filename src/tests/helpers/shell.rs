use super::*;

#[test]
fn an_ordinary_path_is_just_quoted() {
    assert_eq!(quote("/usr/bin/atrium"), "'/usr/bin/atrium'");
}

#[test]
fn a_space_stays_one_word() {
    assert_eq!(quote("/home/a b/atrium"), "'/home/a b/atrium'");
}

#[test]
fn a_quote_is_closed_escaped_and_reopened() {
    assert_eq!(quote("/tmp/ev'il"), r"'/tmp/ev'\''il'");
}

#[test]
fn a_path_with_both_cannot_break_out() {
    // The pair claude's exec form makes impossible, and the one this has to
    // survive: the shell sees one word, and nothing in it is a command.
    assert_eq!(quote("/tmp/a b'; rm -rf /; echo '"), r"'/tmp/a b'\''; rm -rf /; echo '\'''");
}

#[test]
fn nothing_inside_quotes_is_expanded() {
    assert_eq!(quote("$HOME/`whoami`/${x}"), "'$HOME/`whoami`/${x}'");
}
