use std::{
    fs, io,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::SystemTime,
};

use portable_pty::CommandBuilder;

use crate::{
    adapters::{self, Activity, AgentKind, KeyMeaning, StatusSource, Wiring},
    core::{
        git::{self, GitContext},
        profile::{self, Profile},
        pty::PtySession,
    },
    helpers::palette::Theme,
    ipc::wire::{self, Report},
};

/// Agent ids only have to be unique within one atrium, which a counter gives.
static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// How late a hook's stamp runs: it is taken once the hook's process is up,
/// which is after the moment it describes. Read this much earlier, a turn's
/// late `Stop` loses to the next turn starting instead of ending it.
const HOOK_SLACK_MS: u64 = 100;

/// How long a turn that looks finished from the outside waits for the hook
/// that says how it finished. Nothing arriving means it was stopped.
const SETTLE_MS: u64 = 1_500;

/// A `Stop` this soon after a failure is the same turn ending: copilot says
/// that a turn failed and then that it stopped.
const FAILED_STOP_MS: u64 = 2_000;

/// A tool starting this long after a key was taken for an interrupt proves the
/// turn never stopped. Anything sooner may have been on its way already.
const REVIVE_MS: u64 = 1_000;

/// A session file stamped this long before the agent was spawned was written by
/// a process that had its pid before it.
const STALE_MS: u64 = 1_000;

/// The word for a turn that was stopped rather than finished. codex fires it,
/// opencode's plugin sends it, and a key can mean it.
pub const INTERRUPT: &str = "Interrupt";

/// What an agent is doing.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    /// Nothing going on: fresh, or stopped by you.
    Idle,
    Working,
    NeedsInput,
    /// Its turn ended, and the answer is there to be read.
    Done,
    Error,
    Exited,
}

impl Status {
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Idle | Self::Done => "○",
            Self::Working => "◐",
            Self::NeedsInput => "●",
            Self::Error => "✗",
            Self::Exited => "·",
        }
    }

    /// The status a hook event means. Unknown events are ignored rather than
    /// guessed at. `SessionEnd` is one of them: claude and copilot both fire it
    /// on `/clear`, so only the child going away means an agent has exited.
    pub fn from_hook_event(event: &str) -> Option<Self> {
        match event {
            "SessionStart" => Some(Self::Idle),
            "UserPromptSubmit" => Some(Self::Working),
            "Notification" | "PermissionRequest" | "Elicitation" => Some(Self::NeedsInput),
            "Stop" => Some(Self::Done),
            "StopFailure" => Some(Self::Error),
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
        // These lift a wait and never set one. A prompt being answered is
        // announced by nothing, so the tool going ahead -- or being refused --
        // is the first word of it; but hooks arrive out of order, and one
        // landing after `Stop` must not pull a finished turn back to working.
        if lifts_a_wait(event) {
            return if self == Self::NeedsInput { Self::Working } else { self };
        }
        // Stopped turns are only ever ones that were still going.
        if event == INTERRUPT {
            return if self.is_active() { Self::Idle } else { self };
        }
        Self::from_hook_event(event).unwrap_or(self)
    }

    /// Still in a turn: the model has it, or it is waiting on you in the middle
    /// of one.
    pub fn is_active(self) -> bool {
        matches!(self, Self::Working | Self::NeedsInput)
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::NeedsInput => "needs you",
            Self::Done => "done",
            Self::Error => "error",
            Self::Exited => "exited",
        }
    }
}

/// The events that end a wait and do nothing else.
fn lifts_a_wait(event: &str) -> bool {
    matches!(event, "PostToolUse" | "PostToolUseFailure" | "PermissionDenied" | "ElicitationResult")
}

/// The words that say how a turn began or ended. They are still believed once a
/// CLI's own account of itself has taken over saying what it is doing between.
fn is_outcome(event: &str) -> bool {
    matches!(event, "SessionStart" | "Stop" | "StopFailure" | INTERRUPT)
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

/// What the status machine remembers between inputs, apart from the status
/// itself. Every time is ms since the epoch.
#[derive(Default)]
struct Track {
    /// The newest input applied. One stamped earlier arrived late, and says
    /// something about a moment that has already been overtaken.
    last_at: u64,
    /// When the agent was spawned.
    born_at: u64,
    /// When the current failure began.
    failed_at: Option<u64>,
    /// When a key made atrium guess the turn was over.
    guessed_at: Option<u64>,
    /// A turn that looks finished, holding on until this moment for the hook
    /// that says how it finished.
    settle_at: Option<u64>,
    /// Set once the CLI's own account of itself has been read. From then on it
    /// says what the agent is doing, and the hooks only say how turns ended.
    self_reported: bool,
    /// Where that account is kept, and when it was last changed as far as a
    /// read has seen.
    file: Option<(PathBuf, Option<SystemTime>)>,
    /// What the window title last said, so only a change counts.
    title: Option<Activity>,
    /// A turn that ended since the app last asked: an error over a finish.
    ended: Option<Status>,
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
    /// True from a turn ending until the agent has been on the stage: the row's
    /// version of a tmux window being visited.
    pub unseen: bool,
    kind: Box<dyn AgentKind>,
    source: StatusSource,
    git: Option<GitContext>,
    session: PtySession,
    track: Track,
}

impl Agent {
    pub fn spawn(spec: &AgentSpec, harness: &Harness, theme: &Theme, rows: u16, cols: u16) -> io::Result<Self> {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let kind = adapters::detect(&spec.program);

        let mut cmd = spec.command();
        let config_dir = profile::config_dir_env(&spec.program).and_then(|key| spec.env.iter().find(|(name, _)| name == key).map(|(_, value)| value.clone()));
        kind.instrument(&mut cmd, &Wiring { exe: harness.exe.clone(), socket: harness.socket.clone(), agent_id: id, theme: *theme, config_dir: config_dir.clone() });

        let born_at = wire::now_ms();
        let session = PtySession::spawn(cmd, rows, cols)?;
        let file = session.pid().and_then(|pid| kind.activity_file(config_dir.as_deref(), pid)).map(|path| (path, None));
        let git = git::context_for(&spec.cwd);
        Ok(Self {
            id,
            name: spec.name(),
            cwd: spec.cwd.clone(),
            program: spec.program.clone(),
            config_dir,
            profile: spec.profile.clone(),
            status: Status::Idle,
            unseen: false,
            source: kind.status_source(),
            kind,
            git,
            session,
            track: Track { born_at, file, ..Track::default() },
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

    /// Everything atrium can see for itself, once a frame: the child going
    /// away, the CLI's own account of what it is doing, and a finished-looking
    /// turn that no hook has explained in time.
    pub fn refresh_status(&mut self, now: u64) {
        if self.status == Status::Exited {
            return;
        }
        if !self.session.is_alive() {
            self.status = Status::Exited;
            return;
        }
        self.read_file(now);
        self.read_title(now);
        if self.track.settle_at.is_some_and(|due| now >= due) {
            self.track.settle_at = None;
            if self.status.is_active() {
                self.become_(Status::Idle, now);
            }
        }
    }

    /// A report from a hook.
    pub fn apply(&mut self, report: &Report, now: u64) {
        let at = report.at.map_or(now, |at| at.saturating_sub(HOOK_SLACK_MS));
        let event = report.event.as_str();
        if self.track.self_reported && !is_outcome(event) {
            return;
        }
        if at < self.track.last_at {
            return;
        }
        self.track.last_at = at;
        self.track.settle_at = None;

        // A tool starting well after a key was taken for an interrupt means the
        // turn went on without it.
        if event == "PreToolUse" {
            if self.status == Status::Idle && self.track.guessed_at.is_some_and(|guessed| at >= guessed + REVIVE_MS) {
                self.become_(Status::Working, at);
            }
            return;
        }
        if event == "Stop" && self.status == Status::Error && self.track.failed_at.is_some_and(|failed| at <= failed + FAILED_STOP_MS) {
            return;
        }
        self.become_(self.status.after(event), at);
    }

    /// A hook word with nothing else to it, stamped now. What tests and anything
    /// else outside the hook feed use to set a status the way a hook would.
    pub fn apply_event(&mut self, event: &str) {
        self.apply(&Report { agent_id: self.id, event: event.to_owned(), at: None }, wire::now_ms());
    }

    /// A key on its way to the agent, as the bytes it is sent as. For a CLI
    /// that fires nothing when a prompt is answered or a turn interrupted, the
    /// key is all there is to go on.
    pub fn on_key(&mut self, bytes: &[u8], now: u64) {
        let Some(meaning) = self.kind.key_meaning(self.status, bytes) else {
            return;
        };
        self.track.last_at = self.track.last_at.max(now);
        self.track.settle_at = None;
        match meaning {
            KeyMeaning::Answer => self.become_(Status::Working, now),
            KeyMeaning::Stop => {
                self.become_(Status::Idle, now);
                self.track.guessed_at = Some(now);
            },
        }
    }

    /// What ended since this was last asked, if anything did.
    pub fn take_ended(&mut self) -> Option<Status> {
        self.track.ended.take()
    }

    pub fn has_exited(&self) -> bool {
        self.status == Status::Exited
    }

    /// The CLI's account of itself, read again whenever its file changes.
    fn read_file(&mut self, now: u64) {
        let Some((path, seen)) = &self.track.file else {
            return;
        };
        let path = path.clone();
        let Ok(modified) = fs::metadata(&path).and_then(|meta| meta.modified()) else {
            return;
        };
        if *seen == Some(modified) {
            return;
        }
        let (Ok(text), Some(pid)) = (fs::read_to_string(&path), self.session.pid()) else {
            return;
        };
        // Half written, most likely. The change is left unrecorded, so the
        // next frame reads it again.
        let Some(reading) = self.kind.read_activity(&text, pid) else {
            return;
        };
        self.track.file = Some((path, Some(modified)));
        if reading.at + STALE_MS < self.track.born_at {
            return;
        }
        self.track.self_reported = true;
        if let Some(activity) = reading.activity {
            self.observe(activity, reading.at, now);
        }
    }

    /// The window title, for a CLI that says what it is doing there.
    fn read_title(&mut self, now: u64) {
        let Some(title) = self.session.take_title() else {
            return;
        };
        let Some(activity) = self.kind.title_activity(&title) else {
            return;
        };
        if self.track.title == Some(activity) {
            return;
        }
        self.track.title = Some(activity);
        self.observe(activity, now, now);
    }

    /// What the CLI itself says it is doing.
    ///
    /// Busy and waiting are believed at once. Idle is not: it looks the same
    /// whether the turn finished, failed or was stopped, and the hook that
    /// tells those apart can land a moment after it -- so it only arms the
    /// settle, and leaves the stamp where it was for that hook to pass.
    fn observe(&mut self, activity: Activity, at: u64, now: u64) {
        if at < self.track.last_at {
            return;
        }
        if activity == Activity::Idle {
            if self.status.is_active() && self.track.settle_at.is_none() {
                self.track.settle_at = Some(now + SETTLE_MS);
            }
            return;
        }
        self.track.last_at = at;
        self.track.settle_at = None;
        self.become_(if activity == Activity::Busy { Status::Working } else { Status::NeedsInput }, at);
    }

    fn become_(&mut self, next: Status, at: u64) {
        if next == self.status {
            return;
        }
        self.track.guessed_at = None;
        if next == Status::Error {
            self.track.failed_at = Some(at);
        }
        if matches!(next, Status::Done | Status::Error) {
            self.track.ended = Some(if self.track.ended == Some(Status::Error) { Status::Error } else { next });
        }
        self.unseen = matches!(next, Status::Done | Status::Error);
        self.status = next;
    }
}

#[cfg(test)]
#[path = "../tests/core/agent.rs"]
mod tests;
