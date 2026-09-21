use std::path::{Path, PathBuf};

use portable_pty::CommandBuilder;
use ratatui::style::Color;

use crate::{
    adapters::{AgentKind, StatusSource, Wiring, tag},
    helpers::{json, palette::Theme},
};

/// The hook events atrium registers, and the status each one means. Registering
/// per event is what lets the handler take the event name as an argument
/// instead of parsing Claude's payload.
pub const HOOK_EVENTS: [&str; 9] = ["SessionStart", "UserPromptSubmit", "Notification", "PermissionRequest", "PostToolUse", "PermissionDenied", "Stop", "StopFailure", "SessionEnd"];

/// Which `Notification`s are worth pulling you over for.
///
/// Claude rings one bell for eleven different things: a permission prompt and a
/// question of its own, but also "you have not typed in a while", "the turn is
/// finished", "you signed in", and three about quota. Registered bare, all
/// eleven read as "needs you" -- so a row turned blue the moment a turn ended
/// and pulsed there until it was answered, which is the one state the colour
/// was supposed to distinguish itself from.
///
/// The matcher tells them apart **in claude**, so the handler still takes its
/// event name as an argument and still reads no payload. Letters, `_` and `|`
/// only, which is claude's exact-match path: these four spellings, and nothing
/// that merely contains one.
const NEEDS_YOU: &str = "permission_prompt|elicitation_dialog|elicitation_url_dialog|agent_needs_input";

/// What an event is narrowed to, for the one event where being told everything
/// is worse than being told some of it.
fn matcher_for(event: &str) -> Option<&'static str> {
    (event == "Notification").then_some(NEEDS_YOU)
}

/// What atrium's theme is filed under. Claude reads a user theme from
/// `<config dir>/themes/<slug>.json` and names it `custom:<slug>`.
const THEME_SLUG: &str = "atrium";

/// Where a claude with no `CLAUDE_CONFIG_DIR` of its own keeps everything.
const DEFAULT_CONFIG_DIR: &str = ".claude";

pub struct ClaudeCode;

impl AgentKind for ClaudeCode {
    fn id(&self) -> &'static str {
        "claude"
    }

    /// Hooks go in through `--settings` rather than `~/.claude/settings.json`,
    /// so a Claude started outside atrium is untouched. The theme rides along
    /// in the same document, because a second `--settings` would replace this
    /// one rather than adding to it.
    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
        let theme = install(wiring.config_dir.as_deref(), &wiring.theme).is_ok().then_some(THEME_SLUG);
        cmd.arg("--settings");
        cmd.arg(settings_json(&wiring.exe.to_string_lossy(), theme));
    }

    /// Claude watches its theme files and repaints, so this reaches an agent
    /// that is already up -- rewriting the file is the whole of it.
    fn retheme(&self, config_dir: Option<&str>, theme: &Theme) {
        let _ = install(config_dir, theme);
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Hooks
    }
}

/// One hook per event, all pointing back at `atrium hook <event>`, and the
/// theme when there is one to name. `Notification` is narrowed to the kinds
/// that mean you -- see `NEEDS_YOU`.
///
/// `args` puts this in exec form, which runs the handler directly instead of
/// through a shell -- so a path containing a space or a quote cannot be
/// re-split or escaped out of. `async` so a hook never sits between Claude and
/// the thing it was doing.
pub fn settings_json(exe: &str, theme: Option<&str>) -> String {
    let exe = json::escape(exe);
    let entries: Vec<String> = HOOK_EVENTS
        .iter()
        .map(|event| {
            let narrowed = matcher_for(event).map(|types| format!(r#""matcher":"{types}","#)).unwrap_or_default();
            format!(r#""{event}":[{{{narrowed}"hooks":[{{"type":"command","command":"{exe}","args":["hook","{event}"],"async":true}}]}}]"#)
        })
        .collect();
    let theme = theme.map(|slug| format!(r#","theme":"custom:{}""#, json::escape(slug))).unwrap_or_default();
    format!(r#"{{"hooks":{{{}}}{theme}}}"#, entries.join(","))
}

/// The colours atrium has an opinion about, paired with the claude key that
/// carries them. Everything else is left to whichever built-in theme `base`
/// names, which is what keeps this a set of **overrides** rather than a theme
/// atrium has to keep complete as claude adds to it.
fn overrides(theme: &Theme) -> [(&'static str, Color); 26] {
    [
        ("text", theme.COLOR_TEXT),
        ("subtle", theme.COLOR_GREY_600),
        ("inactive", theme.COLOR_GREY_600),
        ("suggestion", theme.COLOR_GREY_500),
        ("remember", theme.COLOR_PURPLE),
        ("promptBorder", theme.COLOR_BORDER),
        ("bashBorder", theme.COLOR_PINK),
        ("skill", theme.COLOR_PINK),
        ("autoAccept", theme.COLOR_PINK),
        ("permission", theme.COLOR_BLUE),
        ("planMode", theme.COLOR_CYAN),
        ("ide", theme.COLOR_BLUE),
        // Claude's own mark, kept in the theme's orange rather than atrium's
        // purple: it is whose agent it is, not whose window.
        ("claude", theme.COLOR_ORANGE),
        ("success", theme.COLOR_GREEN),
        ("error", theme.COLOR_RED),
        ("warning", theme.COLOR_AMBER),
        ("diffAdded", theme.COLOR_LIGHT_GREEN_900),
        ("diffRemoved", theme.COLOR_DARK_RED),
        ("diffAddedWord", theme.COLOR_GREEN),
        ("diffRemovedWord", theme.COLOR_RED),
        ("red_FOR_SUBAGENTS_ONLY", theme.COLOR_RED),
        ("blue_FOR_SUBAGENTS_ONLY", theme.COLOR_BLUE),
        ("green_FOR_SUBAGENTS_ONLY", theme.COLOR_GREEN),
        ("yellow_FOR_SUBAGENTS_ONLY", theme.COLOR_YELLOW),
        ("purple_FOR_SUBAGENTS_ONLY", theme.COLOR_PURPLE),
        ("cyan_FOR_SUBAGENTS_ONLY", theme.COLOR_CYAN),
    ]
}

/// One colour the way claude's themes write one: `rgb(r,g,b)`, or `ansi:name`
/// for one of the terminal's own.
///
/// None for anything that cannot be said -- `Reset` above all, which means
/// "whatever the terminal already had" and has no spelling here. The key is
/// then left out and `base` keeps its own answer, which is the nearest thing to
/// leaving it alone.
fn value(color: Color) -> Option<String> {
    match color {
        Color::Rgb(red, green, blue) => Some(format!("rgb({red},{green},{blue})")),
        Color::Black => Some("ansi:black".to_owned()),
        Color::Red => Some("ansi:red".to_owned()),
        Color::Green => Some("ansi:green".to_owned()),
        Color::Yellow => Some("ansi:yellow".to_owned()),
        Color::Blue => Some("ansi:blue".to_owned()),
        Color::Magenta => Some("ansi:magenta".to_owned()),
        Color::Cyan => Some("ansi:cyan".to_owned()),
        Color::Gray => Some("ansi:white".to_owned()),
        Color::DarkGray => Some("ansi:blackBright".to_owned()),
        Color::LightRed => Some("ansi:redBright".to_owned()),
        Color::LightGreen => Some("ansi:greenBright".to_owned()),
        Color::LightYellow => Some("ansi:yellowBright".to_owned()),
        Color::LightBlue => Some("ansi:blueBright".to_owned()),
        Color::LightMagenta => Some("ansi:magentaBright".to_owned()),
        Color::LightCyan => Some("ansi:cyanBright".to_owned()),
        Color::White => Some("ansi:whiteBright".to_owned()),
        _ => None,
    }
}

/// Which built-in theme the overrides sit on top of, so the keys atrium says
/// nothing about still read. Decided by the background's own brightness, and
/// dark for anything that cannot be measured -- the presets are mostly dark and
/// a light theme under dark text is the worse way to be wrong.
pub fn base_for(theme: &Theme) -> &'static str {
    match theme.background_color() {
        // Rec. 601 luma, which is enough to tell a light background from a dark
        // one without carrying a colour library for it.
        Color::Rgb(red, green, blue) => {
            let luma = 0.299 * f32::from(red) + 0.587 * f32::from(green) + 0.114 * f32::from(blue);
            if luma > 128.0 { "light" } else { "dark" }
        },
        Color::White | Color::Gray => "light",
        _ => "dark",
    }
}

/// atrium's palette as a claude user theme.
pub fn theme_json(theme: &Theme) -> String {
    let entries: Vec<String> = overrides(theme).iter().filter_map(|(key, colour)| Some(format!("    \"{key}\": \"{}\"", value(*colour)?))).collect();
    format!("{{\n  \"name\": \"atrium\",\n  \"base\": \"{}\",\n  \"overrides\": {{\n{}\n  }}\n}}\n", base_for(theme), entries.join(",\n"))
}

/// Where the theme goes for an agent launched against `config_dir`. A profile
/// that names no directory is a claude using its own default, which is
/// `~/.claude` -- the same rule claude applies to itself.
pub fn theme_path(config_dir: Option<&str>) -> PathBuf {
    let base = match config_dir {
        Some(dir) => PathBuf::from(dir),
        None => dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(DEFAULT_CONFIG_DIR),
    };
    base.join("themes").join(format!("{THEME_SLUG}.json"))
}

/// Writes it, and says where. One file per subscription, because which
/// `CLAUDE_CONFIG_DIR` is in use is which directory claude reads themes from.
pub fn install(config_dir: Option<&str>, theme: &Theme) -> std::io::Result<PathBuf> {
    let path = theme_path(config_dir);
    install_to(&path, theme)?;
    Ok(path)
}

pub fn install_to(path: &Path, theme: &Theme) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, theme_json(theme))
}

#[cfg(test)]
#[path = "../tests/adapters/claude.rs"]
mod tests;
