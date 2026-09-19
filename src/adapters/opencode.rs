use portable_pty::CommandBuilder;

use crate::adapters::{AgentKind, StatusSource, Wiring, tag};

/// Held, but it does not report back yet -- it is tagged so that it can once
/// opencode grows something hook-shaped.
pub struct OpenCode;

impl AgentKind for OpenCode {
    fn id(&self) -> &'static str {
        "opencode"
    }

    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Heuristic
    }
}
