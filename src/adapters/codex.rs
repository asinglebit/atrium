use portable_pty::CommandBuilder;

use crate::{
    adapters::{Activity, AgentKind, StatusSource, Wiring, tag},
    helpers::{json, shell},
};

/// The hooks atrium hands codex. Only the three that say how a turn began and
/// ended: what happens in between is in codex's title, which knows about every
/// approval and question -- its `PermissionRequest` also fires for calls it then
/// approves by itself, and nothing fires for an answer.
pub const HOOK_EVENTS: [&str; 3] = ["UserPromptSubmit", "Stop", "Interrupt"];

/// How long codex waits for one of these. The handler writes one line to a
/// socket and exits; codex's own default is ten minutes.
const TIMEOUT_SECONDS: u32 = 5;

/// The title codex is asked to keep: what it is doing, and nothing else to get
/// in the way of reading it.
const TITLE: &str = r#"tui.terminal_title=["activity","run-state"]"#;

/// Fully wired, through `-c` on the command line and the title it writes.
pub struct Codex;

impl AgentKind for Codex {
    fn id(&self) -> &'static str {
        "codex"
    }

    /// `-c` sets a config key for this process alone, so `~/.codex/config.toml`
    /// is never written -- the same trade claude's `--settings` makes, with one
    /// difference: a hook of your own in `config.toml` for one of these events
    /// is replaced rather than joined for as long as this agent runs.
    ///
    /// codex asks, the first time, whether to trust hooks it has not seen. These
    /// are written the same way every time, so it asks once -- until atrium runs
    /// from a different path.
    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
        cmd.args(config_args(&wiring.exe.to_string_lossy()));
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Hooks
    }

    fn title_activity(&self, title: &str) -> Option<Activity> {
        title_activity(title)
    }
}

/// Every `-c`, as the words codex is handed. Each value is TOML; the command in
/// it is a shell string, so the path is quoted for the shell first and escaped
/// for TOML after.
pub fn config_args(exe: &str) -> Vec<String> {
    let command = shell::quote(exe);
    let mut args = Vec::new();
    for event in HOOK_EVENTS {
        let hook = json::escape(&format!("{command} hook {event}"));
        args.push("-c".to_owned());
        args.push(format!(r#"hooks.{event}=[{{hooks=[{{type="command",command="{hook}",timeout={TIMEOUT_SECONDS}}}]}}]"#));
    }
    args.push("-c".to_owned());
    args.push(TITLE.to_owned());
    args
}

/// What codex's title says, given `activity` and `run-state`:
/// `[ ! ] Action Required` -- blinking with `[ . ]` -- while anything waits on
/// you; a spinner and `Working`, `Thinking` or `Waiting` while it works; and
/// `Ready` once the turn is over, however it ended.
pub fn title_activity(title: &str) -> Option<Activity> {
    if title.contains("Action Required") {
        Some(Activity::Waiting)
    } else if ["Working", "Thinking", "Waiting"].iter().any(|word| title.contains(word)) {
        Some(Activity::Busy)
    } else if title.contains("Ready") {
        Some(Activity::Idle)
    } else {
        None
    }
}

#[cfg(test)]
#[path = "../tests/adapters/codex.rs"]
mod tests;
