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

#[test]
fn the_plugin_is_named_by_an_absolute_path_in_json() {
    let content = config_content(std::path::Path::new(r#"/home/a "b"/atrium.js"#));
    assert_eq!(content, r#"{"plugin":["/home/a \"b\"/atrium.js"]}"#);
}

#[test]
fn a_config_of_your_own_is_never_replaced() {
    let mut cmd = CommandBuilder::new("opencode");
    cmd.env_remove(CONFIG_CONTENT_ENV);
    assert!(hands_over_plugin(&cmd));

    cmd.env(CONFIG_CONTENT_ENV, r#"{"model":"mine"}"#);
    assert!(!hands_over_plugin(&cmd), "replacing it would drop everything it says");
}

#[test]
fn the_plugin_lives_with_atriums_own_files() {
    assert!(plugin_path().ends_with("atrium/opencode-plugin/atrium.js"), "{}", plugin_path().display());
}

#[test]
fn installing_the_plugin_writes_it_whole() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("opencode-plugin").join("atrium.js");

    let written = install_plugin_to(&path).expect("install");

    assert_eq!(written, path);
    assert_eq!(std::fs::read_to_string(&path).expect("plugin"), PLUGIN);
}

#[test]
fn every_word_the_plugin_sends_is_one_atrium_hears() {
    use crate::core::agent::{INTERRUPT, Status};
    for word in ["UserPromptSubmit", "PermissionRequest", "PostToolUse", "Stop", "StopFailure", INTERRUPT] {
        assert!(PLUGIN.contains(&format!("\"{word}\"")), "{word} is no longer sent -- or is sent under another name");
        let heard = Status::from_hook_event(word).is_some() || Status::NeedsInput.after(word) != Status::NeedsInput || Status::Working.after(word) != Status::Working;
        assert!(heard, "{word} would reach atrium and mean nothing");
    }
}

#[test]
fn the_plugin_forgets_how_to_reach_atrium_once_it_has() {
    // An opencode one of this one's tools starts would otherwise load the
    // plugin too, and report as the agent that started it.
    for name in ["ATRIUM_SOCK", "ATRIUM_AGENT_ID", CONFIG_CONTENT_ENV] {
        assert!(PLUGIN.contains(&format!("delete process.env.{name}")), "{name} is left for children to inherit");
    }
}

#[test]
fn the_plugin_keeps_one_connection() {
    assert_eq!(PLUGIN.matches("net.createConnection(").count(), 1, "separate connections would not keep the lines in order");
    assert!(PLUGIN.contains("connection.unref()"), "nor should it keep opencode running once it wants to exit");
    assert!(PLUGIN.is_ascii(), "the socket's reader stops for good at a line that is not UTF-8, so nothing here should come close");
}
