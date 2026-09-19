pub mod claude;
pub mod codex;
pub mod opencode;

use std::path::PathBuf;

use portable_pty::CommandBuilder;

use crate::helpers::palette::Theme;

/// Where an agent's status comes from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StatusSource {
    /// The CLI calls back into atrium and says what it is doing.
    Hooks,
    /// Nothing calls back, so atrium can only watch the process.
    Heuristic,
}

/// What atrium hands an agent when it is launched: the way home, and the look.
#[derive(Clone, Debug)]
pub struct Wiring {
    pub exe: PathBuf,
    pub socket: PathBuf,
    pub agent_id: u64,
    pub theme: Theme,
    /// Where this agent keeps its own configuration, when a profile said so.
    /// A claude subscription's themes live inside it, so which subscription is
    /// in use decides where the theme has to be written.
    pub config_dir: Option<String>,
}

pub trait AgentKind {
    fn id(&self) -> &'static str;

    /// Adds whatever makes this CLI report its status back to atrium.
    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring);

    /// Rewrites whatever this CLI reads its colours from, for an agent that is
    /// **already running**. Doing nothing is the default: most CLIs cannot be
    /// told, and one that keeps its own colours is not a failure.
    ///
    /// Whether it lands while the agent is up is the CLI's business. claude
    /// watches the file and repaints; opencode reads once and keeps what it
    /// started with, so for that one this is only ever for the next agent.
    fn retheme(&self, _config_dir: Option<&str>, _theme: &Theme) {}

    fn status_source(&self) -> StatusSource;
}

/// Picks the adapter from the program name, ignoring any directories around it.
pub fn detect(program: &str) -> Box<dyn AgentKind> {
    let name = program.rsplit('/').next().unwrap_or(program);
    match name {
        "claude" => Box::new(claude::ClaudeCode),
        "opencode" => Box::new(opencode::OpenCode),
        "codex" => Box::new(codex::Codex),
        _ => Box::new(Unknown),
    }
}

/// Anything atrium does not recognise still gets held; it just cannot say more
/// than whether it is alive.
pub struct Unknown;

impl AgentKind for Unknown {
    fn id(&self) -> &'static str {
        "unknown"
    }

    fn instrument(&self, cmd: &mut CommandBuilder, wiring: &Wiring) {
        tag(cmd, wiring);
    }

    fn status_source(&self) -> StatusSource {
        StatusSource::Heuristic
    }
}

/// Every agent carries its identity and the way home, whether or not its CLI
/// knows how to use them.
pub fn tag(cmd: &mut CommandBuilder, wiring: &Wiring) {
    cmd.env("ATRIUM_AGENT_ID", wiring.agent_id.to_string());
    cmd.env("ATRIUM_SOCK", &wiring.socket);
}
