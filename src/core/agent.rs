use std::{
    io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use portable_pty::CommandBuilder;

use crate::{
    adapters::{self, StatusSource, Wiring},
    core::{
        git::{self, GitContext},
        profile::{self, Profile},
        pty::PtySession,
    },
    helpers::palette::Theme,
};

/// Agent ids only have to be unique within one atrium, which a counter gives.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// What an agent is doing. Only `Idle` and `Exited` are produced for now; the
/// rest are what the hook feed will set once it exists.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Idle,
    Working,
    NeedsInput,
    Error,
    Exited,
}

impl Status {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Idle => "○",
            Self::Working => "◐",
            Self::NeedsInput => "●",
            Self::Error => "✗",
            Self::Exited => "·",
        }
    }

    /// The status a Claude hook event means. Unknown events are ignored rather
    /// than guessed at.
    pub fn from_hook_event(event: &str) -> Option<Self> {
        match event {
            "SessionStart" | "Stop" => Some(Self::Idle),
            "UserPromptSubmit" => Some(Self::Working),
            "Notification" | "PermissionRequest" => Some(Self::NeedsInput),
            "StopFailure" => Some(Self::Error),
            "SessionEnd" => Some(Self::Exited),
            _ => None,
        }
    }

    /// What this status becomes when `event` arrives.
    ///
    /// An agent that has already exited stays exited: a hook that arrives late
    /// must not bring a dead row back to life.
    pub fn after(self, event: &str) -> Self {
        if self == Self::Exited {
            return self;
        }
        // These two lift a wait and never set one. Claude says nothing when a
        // permission prompt is answered, so the tool going ahead -- or being
        // refused -- is the first word of it; but hooks arrive out of order,
        // and one landing after `Stop` must not pull a finished turn back to
        // working.
        if matches!(event, "PostToolUse" | "PermissionDenied") {
            return if self == Self::NeedsInput { Self::Working } else { self };
        }
        Self::from_hook_event(event).unwrap_or(self)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::NeedsInput => "needs you",
            Self::Error => "error",
            Self::Exited => "exited",
        }
    }
}

/// Everything needed to start an agent, kept instead of a `CommandBuilder` so
/// that "another one of these" is possible -- CommandBuilder cannot be cloned.
#[derive(Clone, Debug)]
pub struct AgentSpec {
    pub program: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    /// Added to the child's environment. A Claude subscription is exactly this:
    /// a `CLAUDE_CONFIG_DIR` pointing somewhere other than the default.
    pub env: Vec<(String, String)>,
    /// The profile this came from, for the sidebar row. None when it did not
    /// come from a named one.
    pub profile: Option<String>,
}

impl AgentSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>, cwd: impl Into<PathBuf>) -> Self {
        Self { program: program.into(), args, cwd: cwd.into(), env: Vec::new(), profile: None }
    }

    pub fn from_profile(profile: &Profile, cwd: impl Into<PathBuf>) -> Self {
        Self { program: profile.program.clone(), args: profile.args.clone(), cwd: cwd.into(), env: profile.env.clone(), profile: profile.tag() }
    }

    pub fn command(&self) -> CommandBuilder {
        let mut cmd = CommandBuilder::new(&self.program);
        cmd.args(&self.args);
        cmd.cwd(&self.cwd);
        // Before the adapter's own wiring, so a profile cannot shadow ATRIUM_*.
        for (key, value) in &self.env {
            cmd.env(key, value);
        }
        cmd
    }

    /// The directory's own name is what identifies an agent in the sidebar; the
    /// program name only helps when the path has no last component.
    pub fn name(&self) -> String {
        self.cwd.file_name().and_then(|n| n.to_str()).map(str::to_owned).unwrap_or_else(|| self.program.clone())
    }
}

/// The parts of the wiring that are identical for every agent in one atrium.
#[derive(Clone, Debug)]
pub struct Harness {
    pub exe: PathBuf,
    pub socket: PathBuf,
}

pub struct Agent {
    pub id: u64,
    pub name: String,
    pub cwd: PathBuf,
    /// What it was launched as, which is what picks the adapter again when the
    /// theme changes under a running agent.
    pub program: String,
    /// The configuration directory it was launched against, when a profile
    /// named one -- where a claude subscription keeps its themes.
    pub config_dir: Option<String>,
    /// Which profile it was held under, when that says something a bare CLI
    /// name would not.
    pub profile: Option<String>,
    pub status: Status,
    source: StatusSource,
    git: Option<GitContext>,
    session: PtySession,
}

impl Agent {
    pub fn spawn(spec: &AgentSpec, harness: &Harness, theme: &Theme, rows: u16, cols: u16) -> io::Result<Self> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let kind = adapters::detect(&spec.program);

        let mut cmd = spec.command();
        let config_dir = spec.env.iter().find(|(key, _)| key == profile::CONFIG_DIR_ENV).map(|(_, value)| value.clone());
        kind.instrument(&mut cmd, &Wiring { exe: harness.exe.clone(), socket: harness.socket.clone(), agent_id: id, theme: *theme, config_dir: config_dir.clone() });

        let session = PtySession::spawn(cmd, rows, cols)?;
        let git = git::context_for(&spec.cwd);
        Ok(Self {
            id,
            name: spec.name(),
            cwd: spec.cwd.clone(),
            program: spec.program.clone(),
            config_dir,
            profile: spec.profile.clone(),
            status: Status::Idle,
            source: kind.status_source(),
            git,
            session,
        })
    }

    pub fn git(&self) -> Option<&GitContext> {
        self.git.as_ref()
    }

    /// Re-read the branch and whether the tree is dirty. Called on a timer, not
    /// every frame: status on a large repo is far too slow for the draw loop.
    pub fn refresh_git(&mut self) {
        self.git = git::context_for(&self.cwd);
    }

    /// Whether this agent can say what it is doing, or only whether it is alive.
    pub fn status_source(&self) -> StatusSource {
        self.source
    }

    pub fn session(&self) -> &PtySession {
        &self.session
    }

    pub fn cwd(&self) -> &Path {
        &self.cwd
    }

    pub fn resize(&mut self, rows: u16, cols: u16) -> io::Result<()> {
        self.session.resize(rows, cols)
    }

    pub fn write(&mut self, bytes: &[u8]) -> io::Result<()> {
        self.session.write(bytes)
    }

    /// A dead child is the only status change atrium can see on its own; the
    /// hook feed supplies the rest.
    pub fn refresh_status(&mut self) {
        if !self.session.is_alive() {
            self.status = Status::Exited;
        }
    }

    pub fn apply_event(&mut self, event: &str) {
        self.status = self.status.after(event);
    }

    pub fn has_exited(&self) -> bool {
        self.status == Status::Exited
    }
}

#[cfg(test)]
#[path = "../tests/core/agent.rs"]
mod tests;
