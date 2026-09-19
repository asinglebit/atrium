use std::{
    io,
    path::{Path, PathBuf},
};

use portable_pty::CommandBuilder;
use ratatui::style::Color;

use crate::{
    adapters::{AgentKind, StatusSource, Wiring, tag},
    helpers::palette::Theme,
};

/// What the theme atrium writes is called. A name of its own, so nothing
/// opencode or you already keep under that directory is ever overwritten.
const THEME_NAME: &str = "atrium";

/// The environment variable that hands opencode a TUI config of atrium's own.
/// It is read **in addition to** opencode's own `tui.json`, so everything else
/// that file says -- keybinds, scroll speed -- is left alone.
const TUI_CONFIG_ENV: &str = "OPENCODE_TUI_CONFIG";

/// Held, but it does not report back yet -- it is tagged so that it can once
/// opencode grows something hook-shaped. What it does take from atrium is the
/// theme.
pub struct OpenCode;

impl AgentKind for OpenCode {
    fn id(&self) -> &'static str {
        "opencode"
    }

    /// A theme that cannot be written is passed over: an opencode wearing its
    /// own colours is still an opencode, and refusing to launch one over that
    /// would be the wrong trade.
    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
        if let Ok(path) = install(&wiring.theme) {
            cmd.env(TUI_CONFIG_ENV, path);
        }
    }

    /// Written for the next opencode rather than this one: opencode reads the
    /// theme once at startup and watches neither file -- both were tried.
    fn retheme(&self, _config_dir: Option<&str>, theme: &Theme) {
        let _ = install(theme);
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Heuristic
    }
}

/// Every colour opencode's themes name, paired with the atrium colour that
/// stands for it. Written out in full rather than left to opencode's own
/// fallbacks, so a row atrium has an opinion about carries it.
fn palette(theme: &Theme) -> [(&'static str, Color); 50] {
    [
        ("primary", theme.COLOR_PURPLE),
        ("secondary", theme.COLOR_PINK),
        ("accent", theme.COLOR_GRASS),
        ("error", theme.COLOR_RED),
        ("warning", theme.COLOR_AMBER),
        ("success", theme.COLOR_GREEN),
        ("info", theme.COLOR_BLUE),
        ("text", theme.COLOR_TEXT),
        ("textMuted", theme.COLOR_GREY_600),
        ("background", theme.background_color()),
        ("backgroundPanel", theme.COLOR_GREY_900),
        ("backgroundElement", theme.COLOR_GREY_800),
        ("border", theme.COLOR_BORDER),
        ("borderActive", theme.COLOR_HIGHLIGHTED),
        ("borderSubtle", theme.COLOR_GREY_800),
        ("diffAdded", theme.COLOR_GREEN),
        ("diffRemoved", theme.COLOR_RED),
        ("diffContext", theme.COLOR_GREY_600),
        ("diffHunkHeader", theme.COLOR_GREY_600),
        ("diffHighlightAdded", theme.COLOR_GRASS),
        ("diffHighlightRemoved", theme.COLOR_GRAPEFRUIT),
        ("diffAddedBg", theme.COLOR_LIGHT_GREEN_900),
        ("diffRemovedBg", theme.COLOR_DARK_RED),
        ("diffContextBg", theme.background_color()),
        ("diffLineNumber", theme.COLOR_GREY_600),
        ("diffAddedLineNumberBg", theme.COLOR_LIGHT_GREEN_900),
        ("diffRemovedLineNumberBg", theme.COLOR_DARK_RED),
        ("markdownText", theme.COLOR_TEXT),
        ("markdownHeading", theme.COLOR_PURPLE),
        ("markdownLink", theme.COLOR_BLUE),
        ("markdownLinkText", theme.COLOR_CYAN),
        ("markdownCode", theme.COLOR_GRASS),
        ("markdownBlockQuote", theme.COLOR_GREY_600),
        ("markdownEmph", theme.COLOR_AMBER),
        ("markdownStrong", theme.COLOR_PINK),
        ("markdownHorizontalRule", theme.COLOR_GREY_600),
        ("markdownListItem", theme.COLOR_PURPLE),
        ("markdownListEnumeration", theme.COLOR_PURPLE),
        ("markdownImage", theme.COLOR_BLUE),
        ("markdownImageText", theme.COLOR_CYAN),
        ("markdownCodeBlock", theme.COLOR_TEXT),
        ("syntaxComment", theme.COLOR_GREY_600),
        ("syntaxKeyword", theme.COLOR_PINK),
        ("syntaxFunction", theme.COLOR_PURPLE),
        ("syntaxVariable", theme.COLOR_TEXT),
        ("syntaxString", theme.COLOR_GREEN),
        ("syntaxNumber", theme.COLOR_AMBER),
        ("syntaxType", theme.COLOR_CYAN),
        ("syntaxOperator", theme.COLOR_PINK),
        ("syntaxPunctuation", theme.COLOR_TEXT),
    ]
}

/// One colour, the way opencode's theme format takes it: a hex string, or a
/// bare number for one of the terminal's own sixteen.
///
/// `Reset` is `"none"`, which opencode draws as transparent -- the same answer
/// atrium's own themes mean by it: whatever the terminal already had.
fn value(color: Color) -> String {
    match color {
        Color::Rgb(red, green, blue) => format!("\"#{red:02x}{green:02x}{blue:02x}\""),
        Color::Reset => "\"none\"".to_owned(),
        Color::Indexed(index) => index.to_string(),
        named => ansi_index(named).to_string(),
    }
}

/// A named terminal colour as its index, which is what opencode's parser takes
/// a bare number to be. atrium's `ansi` preset is built from these, and a hex
/// approximation of it would be a different theme wearing its name.
fn ansi_index(color: Color) -> u8 {
    match color {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::DarkGray => 8,
        Color::LightRed => 9,
        Color::LightGreen => 10,
        Color::LightYellow => 11,
        Color::LightBlue => 12,
        Color::LightMagenta => 13,
        Color::LightCyan => 14,
        _ => 15,
    }
}

/// atrium's palette as an opencode theme file.
pub fn theme_json(theme: &Theme) -> String {
    let colours: Vec<String> = palette(theme).iter().map(|(key, colour)| format!("    \"{key}\": {}", value(*colour))).collect();
    format!("{{\n  \"$schema\": \"https://opencode.ai/theme.json\",\n  \"theme\": {{\n{}\n  }}\n}}\n", colours.join(",\n"))
}

/// The file that picks it. Only the theme, so opencode's own `tui.json` keeps
/// saying everything else it said.
pub fn tui_config_json() -> String {
    format!("{{\n  \"$schema\": \"https://opencode.ai/tui.json\",\n  \"theme\": \"{THEME_NAME}\"\n}}\n")
}

/// Where opencode looks for a theme by name, which is its **own** config
/// directory and nowhere else: not a path written in the config, and not
/// `OPENCODE_CONFIG_DIR`. So this one file is written outside atrium's own
/// directory, and it is the only one.
pub fn theme_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("opencode");
    path.push("themes");
    path.push(format!("{THEME_NAME}.json"));
    path
}

/// The selecting file, which atrium keeps with its own.
pub fn tui_config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("atrium");
    path.push("opencode-tui.json");
    path
}

/// Writes both and hands back the one to point opencode at. Rewritten on every
/// spawn rather than only when missing, because the theme it carries is
/// whichever one the settings last chose.
pub fn install(theme: &Theme) -> io::Result<PathBuf> {
    install_to(&theme_path(), &tui_config_path(), theme)
}

/// The writing, with both destinations handed in -- so it can be tested
/// without writing into the opencode you actually use.
pub fn install_to(theme_path: &Path, config_path: &Path, theme: &Theme) -> io::Result<PathBuf> {
    write(theme_path, &theme_json(theme))?;
    write(config_path, &tui_config_json())?;
    Ok(config_path.to_path_buf())
}

fn write(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, contents)
}

#[cfg(test)]
#[path = "../tests/adapters/opencode.rs"]
mod tests;
