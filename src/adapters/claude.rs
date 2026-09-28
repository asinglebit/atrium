use std::path::{Path, PathBuf};

use facet::Facet;
use portable_pty::CommandBuilder;
use ratatui::style::Color;

use crate::{
    adapters::{Activity, AgentKind, Reading, StatusSource, Wiring, tag},
    helpers::{json, palette::Theme},
};

/// The hook events atrium registers. Registering per event is what lets the
/// handler take the event name as an argument instead of parsing Claude's
/// payload.
///
/// Three of them say how a turn began or ended -- `SessionStart`, `Stop`,
/// `StopFailure` -- and those are always believed. The rest say what happens
/// in between, which claude's session file says better once it has been read:
/// it knows when an approval goes through and when a turn is interrupted, and
/// no hook fires for either. They stay registered for a claude that stops
/// writing the file.
///
/// `SessionEnd` is not here. It fires on `/clear` and on resume as well as on
/// the way out, and only the child going away means an agent has exited.
pub const HOOK_EVENTS: [&str; 11] =
    ["SessionStart", "UserPromptSubmit", "PermissionRequest", "Elicitation", "ElicitationResult", "Notification", "PostToolUse", "PostToolUseFailure", "PermissionDenied", "Stop", "StopFailure"];

/// Which `Notification`s are worth pulling you over for.
///
/// Claude rings one bell for eleven different things: a permission prompt and a
/// question of its own, but also "you have not typed in a while", "the turn is
/// finished", "you signed in", and three about quota. Registered bare, all
/// eleven read as "needs you" -- so a row took the colour of a question the
/// moment a turn ended, and kept it until it was answered, which is the one
/// state that colour was supposed to distinguish itself from.
///
/// The matcher tells them apart **in claude**, so the handler still takes its
/// event name as an argument and still reads no payload. Letters, `_` and `|`
/// only, which is claude's exact-match path: these four spellings, and nothing
/// that merely contains one.
const NEEDS_YOU: &str = "permission_prompt|elicitation_dialog|elicitation_url_dialog|agent_needs_input";

/// Which `SessionStart`s mean a fresh session. `compact` is the one left out: it
/// fires after a compaction, which can happen in the middle of a turn.
const FRESH: &str = "startup|resume|clear|fork";

/// What an event is narrowed to, for the events where being told everything is
/// worse than being told some of it.
pub fn matcher_for(event: &str) -> Option<&'static str> {
    match event {
        "Notification" => Some(NEEDS_YOU),
        "SessionStart" => Some(FRESH),
        _ => None,
    }
}

/// What atrium's theme is filed under. Claude reads a user theme from
/// `<config dir>/themes/<slug>.json` and names it `custom:<slug>`.
const THEME_SLUG: &str = "atrium";

/// Where a claude with no `CLAUDE_CONFIG_DIR` of its own keeps everything.
const DEFAULT_CONFIG_DIR: &str = ".claude";

/// Where claude keeps each running session's account of itself, as
/// `<pid>.json`, beside the rest of its configuration.
const SESSIONS_DIR: &str = "sessions";

/// The one `waitingFor` that is not claude waiting on you: a menu of your own,
/// `/model` or `/config`, open between turns or in the middle of one.
const OWN_MENU: &str = "dialog open";

/// The part of claude's session file that says what it is doing. It rewrites
/// the file on every change, and says `busy`, `waiting` or `idle`. Undocumented,
/// so anything else in it is left unread.
#[derive(Facet)]
#[facet(rename_all = "camelCase")]
struct SessionFile {
    pid: u32,
    status: String,
    #[facet(default)]
    waiting_for: Option<String>,
    status_updated_at: u64,
}

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

    fn activity_file(&self, config_dir: Option<&str>, pid: u32) -> Option<PathBuf> {
        Some(session_path(config_dir, pid))
    }

    fn read_activity(&self, text: &str, pid: u32) -> Option<Reading> {
        read_session(text, pid)
    }
}

/// Where the claude with this pid, launched against `config_dir`, says what it
/// is doing.
pub fn session_path(config_dir: Option<&str>, pid: u32) -> PathBuf {
    config_base(config_dir).join(SESSIONS_DIR).join(format!("{pid}.json"))
}

/// One read of a session file. None when it is not this claude's, or not in a
/// shape atrium knows -- which is also how a half-written one reads.
pub fn read_session(text: &str, pid: u32) -> Option<Reading> {
    let file = facet_json::from_str::<SessionFile>(text).ok()?;
    if file.pid != pid {
        return None;
    }
    let activity = match file.status.as_str() {
        "busy" => Some(Activity::Busy),
        "waiting" if file.waiting_for.as_deref() == Some(OWN_MENU) => None,
        "waiting" => Some(Activity::Waiting),
        "idle" => Some(Activity::Idle),
        _ => return None,
    };
    Some(Reading { activity, at: file.status_updated_at })
}

/// One hook per event, all pointing back at `atrium hook <event>`, and the
/// theme when there is one to name. `Notification` is narrowed to the kinds
/// that mean you, and `SessionStart` to the ones that mean a fresh session --
/// see `matcher_for`.
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
    config_base(config_dir).join("themes").join(format!("{THEME_SLUG}.json"))
}

/// The directory a claude launched against `config_dir` keeps everything in.
fn config_base(config_dir: Option<&str>) -> PathBuf {
    match config_dir {
        Some(dir) => PathBuf::from(dir),
        None => dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(DEFAULT_CONFIG_DIR),
    }
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
