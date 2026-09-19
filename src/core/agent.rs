use std::{
    io,
    path::{Path, PathBuf},
};

use portable_pty::CommandBuilder;

use crate::core::pty::PtySession;

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
}

impl AgentSpec {
    pub fn new(program: impl Into<String>, args: Vec<String>, cwd: impl Into<PathBuf>) -> Self {
        Self { program: program.into(), args, cwd: cwd.into() }
    }

    pub fn command(&self) -> CommandBuilder {
        let mut cmd = CommandBuilder::new(&self.program);
        cmd.args(&self.args);
        cmd.cwd(&self.cwd);
        cmd
    }

    /// The directory's own name is what identifies an agent in the sidebar; the
    /// program name only helps when the path has no last component.
    pub fn name(&self) -> String {
        self.cwd.file_name().and_then(|n| n.to_str()).map(str::to_owned).unwrap_or_else(|| self.program.clone())
    }
}

pub struct Agent {
    pub name: String,
    pub cwd: PathBuf,
    pub status: Status,
    session: PtySession,
}

impl Agent {
    pub fn spawn(spec: &AgentSpec, rows: u16, cols: u16) -> io::Result<Self> {
        let session = PtySession::spawn(spec.command(), rows, cols)?;
        Ok(Self { name: spec.name(), cwd: spec.cwd.clone(), status: Status::Idle, session })
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

    pub fn has_exited(&self) -> bool {
        self.status == Status::Exited
    }
}
