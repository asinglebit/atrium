use super::*;
use crate::helpers::palette::Theme;

/// Every colour opencode's own themes name. Written out here rather than taken
/// from `palette` so that dropping one is a failure rather than a smaller file.
const KEYS: [&str; 50] = [
    "primary",
    "secondary",
    "accent",
    "error",
    "warning",
    "success",
    "info",
    "text",
    "textMuted",
    "background",
    "backgroundPanel",
    "backgroundElement",
    "border",
    "borderActive",
    "borderSubtle",
    "diffAdded",
    "diffRemoved",
    "diffContext",
    "diffHunkHeader",
    "diffHighlightAdded",
    "diffHighlightRemoved",
    "diffAddedBg",
    "diffRemovedBg",
    "diffContextBg",
    "diffLineNumber",
    "diffAddedLineNumberBg",
    "diffRemovedLineNumberBg",
    "markdownText",
    "markdownHeading",
    "markdownLink",
    "markdownLinkText",
    "markdownCode",
    "markdownBlockQuote",
    "markdownEmph",
    "markdownStrong",
    "markdownHorizontalRule",
    "markdownListItem",
    "markdownListEnumeration",
    "markdownImage",
    "markdownImageText",
    "markdownCodeBlock",
    "syntaxComment",
    "syntaxKeyword",
    "syntaxFunction",
    "syntaxVariable",
    "syntaxString",
    "syntaxNumber",
    "syntaxType",
    "syntaxOperator",
    "syntaxPunctuation",
];

#[test]
fn every_colour_opencode_names_is_written() {
    let json = theme_json(&Theme::classic());

    for key in KEYS {
        assert!(json.contains(&format!("\"{key}\":")), "{key} is missing:\n{json}");
    }
    assert_eq!(json.matches("\": ").count(), KEYS.len() + 2, "one line per colour, plus the schema and the theme block:\n{json}");
}

#[test]
fn an_rgb_colour_is_written_as_hex() {
    let json = theme_json(&Theme::classic());

    assert!(json.contains("\"#"), "a hex colour should be quoted:\n{json}");
    assert!(!json.contains("Rgb"), "a debug rendering would not be a colour:\n{json}");
}

#[test]
fn a_terminal_colour_is_written_as_its_index() {
    // The ansi preset is built from the terminal's own sixteen, which opencode
    // takes as bare numbers -- a hex guess at them would be a different theme.
    assert_eq!(value(Color::Red), "1");
    assert_eq!(value(Color::LightMagenta), "13");
    assert_eq!(value(Color::Indexed(238)), "238");

    let json = theme_json(&Theme::ansi());
    assert!(json.contains("\"error\": 1"), "{json}");
}

#[test]
fn a_colour_left_to_the_terminal_is_none() {
    assert_eq!(value(Color::Reset), "\"none\"", "opencode draws `none` as transparent, which is what Reset means");
}

#[test]
fn the_tui_config_picks_the_theme_atrium_writes() {
    let json = tui_config_json();

    assert!(json.contains(&format!("\"theme\": \"{THEME_NAME}\"")), "{json}");
    assert!(!json.contains("keybinds"), "it should say nothing about what it does not own:\n{json}");
}

#[test]
fn the_theme_lands_where_opencode_looks_and_nothing_else_does() {
    let theme = theme_path();
    let config = tui_config_path();

    assert!(theme.ends_with("opencode/themes/atrium.json"), "{}", theme.display());
    assert!(config.ends_with("atrium/opencode-tui.json"), "the file that picks it is atrium's own: {}", config.display());
}

#[test]
fn installing_writes_the_theme_and_the_file_that_picks_it() {
    let dir = tempfile::tempdir().expect("tempdir");
    let theme = dir.path().join("opencode").join("themes").join("atrium.json");
    let config = dir.path().join("atrium").join("opencode-tui.json");

    let handed = install_to(&theme, &config, &Theme::classic()).expect("install");

    assert_eq!(handed, config, "the path handed back is the one the env var takes");
    assert!(std::fs::read_to_string(&theme).expect("theme").contains("syntaxKeyword"));
    assert!(std::fs::read_to_string(&config).expect("config").contains(THEME_NAME));
}

#[test]
fn a_theme_rewrites_rather_than_accumulating() {
    let dir = tempfile::tempdir().expect("tempdir");
    let theme = dir.path().join("themes").join("atrium.json");
    let config = dir.path().join("opencode-tui.json");

    install_to(&theme, &config, &Theme::classic()).expect("first");
    install_to(&theme, &config, &Theme::matrix()).expect("second");

    let written = std::fs::read_to_string(&theme).expect("theme");
    assert_eq!(written, theme_json(&Theme::matrix()), "the last theme chosen is the whole file");
}
