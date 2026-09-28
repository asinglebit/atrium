use std::{
    io,
    path::{Path, PathBuf},
};

use portable_pty::CommandBuilder;

use crate::{
    adapters::{AgentKind, KeyMeaning, StatusSource, Wiring, tag},
    core::agent::Status,
    helpers::{json, shell, version::VERSION},
};

/// What the generated plugin is called. copilot takes a lowercase name, and
/// this is the name it shows under `/env` and `copilot plugin list`.
const PLUGIN_NAME: &str = "atrium";

/// The directory atrium writes the plugin into, under its own config directory.
const PLUGIN_DIR: &str = "copilot-plugin";

/// One hook atrium registers with copilot: the event copilot fires, what it is
/// narrowed to, and the word atrium already understands for it. The wire
/// vocabulary is spelled the way claude spells it, and `Status::after` is the
/// one table that reads it -- so the translation happens here, on the way out,
/// rather than as a second vocabulary in `core`.
pub struct Hook {
    pub event: &'static str,
    /// A regex copilot tries against the event's own discriminator: the
    /// notification's type, the tool's name. Anchored, so a name that merely
    /// contains one of these is not taken for it.
    pub matcher: Option<&'static str>,
    pub word: &'static str,
}

/// The notifications that mean you: a permission prompt, or copilot asking for
/// more. The rest say a background shell or agent finished, which is nothing
/// to answer.
const NEEDS_YOU: &str = "^(permission_prompt|elicitation_dialog)$";

/// copilot's own tool for asking you a question.
const ASKS_YOU: &str = "^ask_user$";

/// What in an `errorOccurred` payload says the turn is over. copilot reports
/// the errors it gets past too, and a turn that recovered has not failed.
const UNRECOVERABLE: &str = r#""recoverable"[[:space:]]*:[[:space:]]*false"#;

/// Every hook, in the order copilot is handed them. `preToolUse` is there
/// twice: once for the question, and once for any tool at all, which is what
/// tells a turn still going from one Esc stopped.
///
/// `sessionEnd` is not here. It fires on `/clear` as well as on the way out,
/// and only the child going away means an agent has exited.
pub const HOOKS: [Hook; 9] = [
    Hook { event: "sessionStart", matcher: None, word: "SessionStart" },
    Hook { event: "userPromptSubmitted", matcher: None, word: "UserPromptSubmit" },
    Hook { event: "notification", matcher: Some(NEEDS_YOU), word: "Notification" },
    Hook { event: "preToolUse", matcher: Some(ASKS_YOU), word: "PermissionRequest" },
    Hook { event: "preToolUse", matcher: None, word: "PreToolUse" },
    Hook { event: "postToolUse", matcher: None, word: "PostToolUse" },
    Hook { event: "postToolUseFailure", matcher: None, word: "PostToolUse" },
    Hook { event: "agentStop", matcher: None, word: "Stop" },
    Hook { event: "errorOccurred", matcher: None, word: "StopFailure" },
];

/// How long copilot waits for one of these before giving up on it. The handler
/// writes one line to a socket and exits, so this is generous; copilot's own
/// default is thirty seconds and there is no "run it in the background" flag to
/// lean on the way claude has.
const TIMEOUT_SECONDS: u32 = 5;

/// Fully wired, through a plugin atrium generates and hands over on the command
/// line. Keeps its own colours: copilot has colour modes rather than themes,
/// and the setting lives in a file you write.
pub struct Copilot;

impl AgentKind for Copilot {
    fn id(&self) -> &'static str {
        "copilot"
    }

    /// The plugin goes in through `--plugin-dir`, never `~/.copilot`, so a
    /// copilot started outside atrium is untouched -- the same trade claude's
    /// `--settings` makes.
    ///
    /// A plugin that cannot be written is passed over: a copilot atrium can
    /// only watch is still a copilot, and refusing to launch one over that
    /// would be the wrong trade.
    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
        if let Ok(dir) = install(&wiring.exe.to_string_lossy()) {
            cmd.arg("--plugin-dir");
            cmd.arg(dir);
        }
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Hooks
    }

    /// copilot fires nothing when a prompt is answered or a turn stopped, so
    /// the key is the only word of it: Esc or Ctrl-C stops a turn, Esc declines
    /// a prompt, and Enter or a digit answers one. Matched on the bytes as
    /// sent, so Alt+1 -- `ESC 1` -- is neither.
    ///
    /// Choosing "no" with Enter or a digit is the one this gets wrong: it reads
    /// as answered until the next prompt.
    fn key_meaning(&self, status: Status, bytes: &[u8]) -> Option<KeyMeaning> {
        match (status, bytes) {
            (Status::Working, [0x1b] | [0x03]) | (Status::NeedsInput, [0x1b]) => Some(KeyMeaning::Stop),
            (Status::NeedsInput, [b'\r'] | [b'1'..=b'9']) => Some(KeyMeaning::Answer),
            _ => None,
        }
    }
}

/// The manifest. `hooks` names the file beside it, which is the only component
/// atrium ships -- no skills, no agents, no MCP servers.
pub fn plugin_json() -> String {
    format!(
        "{{\n  \"name\": \"{PLUGIN_NAME}\",\n  \"description\": \"Reports what this agent is doing back to the atrium holding it\",\n  \"version\": \"{VERSION}\",\n  \"hooks\": \"hooks.json\"\n}}\n"
    )
}

/// Every hook, grouped under the event copilot fires, all pointing back at
/// `atrium hook <word>`.
///
/// copilot's `command` is a **shell string**, not an argument list, so the path
/// is quoted on the way in -- see `helpers::shell`. Only `bash`: the status
/// channel is a unix socket, so there is no Windows to write a `powershell` arm
/// for.
pub fn hooks_json(exe: &str) -> String {
    let command = shell::quote(exe);
    let mut events: Vec<&str> = Vec::new();
    for hook in &HOOKS {
        if !events.contains(&hook.event) {
            events.push(hook.event);
        }
    }
    let entries: Vec<String> = events
        .iter()
        .map(|event| {
            let handlers: Vec<String> = HOOKS.iter().filter(|hook| hook.event == *event).map(|hook| handler(&command, hook)).collect();
            format!("    \"{event}\": [{}]", handlers.join(", "))
        })
        .collect();
    format!("{{\n  \"version\": 1,\n  \"hooks\": {{\n{}\n  }}\n}}\n", entries.join(",\n"))
}

/// One entry. `errorOccurred`'s payload is read here, by the shell, so that
/// `atrium hook` still never reads one: without `-q` grep drains stdin, and the
/// entry exits 0 whether or not anything was reported.
fn handler(command: &str, hook: &Hook) -> String {
    let run = format!("{command} hook {}", hook.word);
    let bash = if hook.event == "errorOccurred" { format!("if grep -E '{UNRECOVERABLE}' >/dev/null; then {run}; fi; exit 0") } else { run };
    let matcher = hook.matcher.map(|matcher| format!(", \"matcher\": \"{}\"", json::escape(matcher))).unwrap_or_default();
    format!("{{ \"type\": \"command\", \"bash\": \"{}\", \"timeoutSec\": {TIMEOUT_SECONDS}{matcher} }}", json::escape(&bash))
}

/// Where the plugin is kept: with atrium's own files, beside the opencode tui
/// config, rather than anywhere copilot reads by itself.
pub fn plugin_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("atrium");
    path.push(PLUGIN_DIR);
    path
}

/// Writes both files and hands back the directory to point copilot at.
/// Rewritten on every spawn rather than only when missing, because the path it
/// carries is wherever this atrium is installed.
pub fn install(exe: &str) -> io::Result<PathBuf> {
    install_to(&plugin_dir(), exe)
}

/// The writing, with the directory handed in -- so it can be tested without
/// writing into the plugin the copilots on this machine would load.
pub fn install_to(dir: &Path, exe: &str) -> io::Result<PathBuf> {
    std::fs::create_dir_all(dir)?;
    std::fs::write(dir.join("plugin.json"), plugin_json())?;
    std::fs::write(dir.join("hooks.json"), hooks_json(exe))?;
    Ok(dir.to_path_buf())
}

#[cfg(test)]
#[path = "../tests/adapters/copilot.rs"]
mod tests;
