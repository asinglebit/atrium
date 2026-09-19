use portable_pty::CommandBuilder;

use crate::adapters::{AgentKind, StatusSource, Wiring, tag};

/// Held, but it does not report back yet -- see `opencode`.
pub struct Codex;

impl AgentKind for Codex {
    fn id(&self) -> &'static str {
        "codex"
    }

    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Heuristic
    }
}
