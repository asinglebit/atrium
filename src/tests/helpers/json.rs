use super::*;

#[test]
fn ordinary_text_is_untouched() {
    assert_eq!(escape("/usr/bin/atrium"), "/usr/bin/atrium");
}

#[test]
fn quotes_and_backslashes_are_escaped() {
    assert_eq!(escape(r#"a"b"#), r#"a\"b"#);
    assert_eq!(escape(r"a\b"), r"a\\b");
}

#[test]
fn a_windows_style_path_survives() {
    assert_eq!(escape(r"C:\bin\atrium.exe"), r"C:\\bin\\atrium.exe");
}

#[test]
fn control_characters_become_escapes() {
    assert_eq!(escape("a\nb"), r"a\nb");
    assert_eq!(escape("a\tb"), r"a\tb");
    assert_eq!(escape("a\u{1}b"), r"a\u0001b");
}

#[test]
fn non_ascii_is_left_alone_because_json_is_utf8() {
    assert_eq!(escape("проект"), "проект");
}
