use std::{
    io,
    path::{Path, PathBuf},
};

use portable_pty::CommandBuilder;

use crate::{
    adapters::{AgentKind, StatusSource, Wiring, tag},
    helpers::{json, shell, version::VERSION},
};

/// What the generated plugin is called. copilot takes a lowercase name, and
/// this is the name it shows under `/env` and `copilot plugin list`.
const PLUGIN_NAME: &str = "atrium";

/// The directory atrium writes the plugin into, under its own config directory.
const PLUGIN_DIR: &str = "copilot-plugin";

/// copilot's own name for each event, and the word atrium already understands
/// for it. The wire vocabulary is spelled the way claude spells it, and
/// `Status::after` is the one table that reads it -- so the translation happens
/// here, on the way out, rather than as a second vocabulary in `core`.
///
/// `notification` is the important one: it is what copilot raises for a
/// permission prompt or a question of its own, which is atrium's "needs you".
pub const HOOK_EVENTS: [(&str, &str); 8] = [
    ("sessionStart", "SessionStart"),
    ("userPromptSubmitted", "UserPromptSubmit"),
    ("notification", "Notification"),
    ("postToolUse", "PostToolUse"),
    ("postToolUseFailure", "PostToolUse"),
    ("agentStop", "Stop"),
    ("errorOccurred", "StopFailure"),
    ("sessionEnd", "SessionEnd"),
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
}

/// The manifest. `hooks` names the file beside it, which is the only component
/// atrium ships -- no skills, no agents, no MCP servers.
pub fn plugin_json() -> String {
    format!(
        "{{\n  \"name\": \"{PLUGIN_NAME}\",\n  \"description\": \"Reports what this agent is doing back to the atrium holding it\",\n  \"version\": \"{VERSION}\",\n  \"hooks\": \"hooks.json\"\n}}\n"
    )
}

/// One hook per event, all pointing back at `atrium hook <event>`.
///
/// copilot's `command` is a **shell string**, not an argument list, so the path
/// is quoted on the way in -- see `helpers::shell`. Only `bash`: the status
/// channel is a unix socket, so there is no Windows to write a `powershell` arm
/// for.
pub fn hooks_json(exe: &str) -> String {
    let command = shell::quote(exe);
    let entries: Vec<String> = HOOK_EVENTS
        .iter()
        .map(|(event, atrium_event)| {
            let bash = json::escape(&format!("{command} hook {atrium_event}"));
            format!("    \"{event}\": [{{ \"type\": \"command\", \"bash\": \"{bash}\", \"timeoutSec\": {TIMEOUT_SECONDS} }}]")
        })
        .collect();
    format!("{{\n  \"version\": 1,\n  \"hooks\": {{\n{}\n  }}\n}}\n", entries.join(",\n"))
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
