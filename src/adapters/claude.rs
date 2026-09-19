use portable_pty::CommandBuilder;

use crate::{
    adapters::{AgentKind, StatusSource, Wiring, tag},
    helpers::json,
};

/// The hook events atrium registers, and the status each one means. Registering
/// per event is what lets the handler take the event name as an argument
/// instead of parsing Claude's payload.
pub const HOOK_EVENTS: [&str; 7] = ["SessionStart", "UserPromptSubmit", "Notification", "PermissionRequest", "Stop", "StopFailure", "SessionEnd"];

pub struct ClaudeCode;

impl AgentKind for ClaudeCode {
    fn id(&self) -> &'static str {
        "claude"
    }

    /// Hooks go in through `--settings` rather than `~/.claude/settings.json`,
    /// so a Claude started outside atrium is untouched.
    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
        cmd.arg("--settings");
        cmd.arg(settings_json(&wiring.exe.to_string_lossy()));
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Hooks
    }
}

/// One hook per event, all pointing back at `atrium hook <event>`.
///
/// `args` puts this in exec form, which runs the handler directly instead of
/// through a shell -- so a path containing a space or a quote cannot be
/// re-split or escaped out of. `async` so a hook never sits between Claude and
/// the thing it was doing.
pub fn settings_json(exe: &str) -> String {
    let exe = json::escape(exe);
    let entries: Vec<String> = HOOK_EVENTS.iter().map(|event| format!(r#""{event}":[{{"hooks":[{{"type":"command","command":"{exe}","args":["hook","{event}"],"async":true}}]}}]"#)).collect();
    format!(r#"{{"hooks":{{{}}}}}"#, entries.join(","))
}

#[cfg(test)]
#[path = "../tests/adapters/claude.rs"]
mod tests;
