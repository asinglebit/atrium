use std::{
    ffi::OsString,
    process::{Command, Stdio},
};

use crate::core::{agent::Status, registry::Counts};

/// Set by tmux on every pane it owns, which is how atrium knows it is in one.
pub const PANE_ENV: &str = "TMUX_PANE";

/// Set by tmux on every process under a server. Checked beside the pane so a
/// stale `TMUX_PANE` inherited outside tmux cannot aim at someone else's pane.
pub const SERVER_ENV: &str = "TMUX";

/// What this atrium needs, in one word. Read by the window the pane sits in.
pub const STATUS_OPTION: &str = "@atrium_status";

/// How many agents are held and what they are doing, for the bar segment.
pub const AGENTS_OPTION: &str = "@atrium_agents";

/// Which half of the pulse the bar should be drawing. Global rather than set on
/// the pane, because every window that holds a waiting agent beats together and
/// a window format resolves an option up the pane, window, session, global
/// chain to find it.
pub const BLINK_OPTION: &str = "@atrium_blink";

/// The pane atrium is drawing in, when it is drawing in one at all.
pub fn pane() -> Option<String> {
    pane_from(std::env::var_os(SERVER_ENV), std::env::var_os(PANE_ENV))
}

/// Split out so the answer can be tested without writing to the one
/// environment every test in the process shares.
pub fn pane_from(server: Option<OsString>, pane: Option<OsString>) -> Option<String> {
    server.filter(|server| !server.is_empty())?;
    pane.filter(|pane| !pane.is_empty()).map(|pane| pane.to_string_lossy().into_owned())
}

/// One word per status, hyphenated rather than spaced: this is matched against
/// inside a tmux format string, where `label()`'s "needs you" would not
/// survive the split.
pub fn word(status: Status) -> &'static str {
    match status {
        Status::Idle => "idle",
        Status::Working => "working",
        Status::NeedsInput => "needs-input",
        Status::Error => "error",
        Status::Exited => "exited",
    }
}

/// The four numbers the bar segment reads, in the order it reads them.
pub fn agents(counts: Counts) -> String {
    format!("{} {} {} {}", counts.held, counts.working, counts.needs_input, counts.error)
}

/// Every option in one invocation, separated the way tmux separates commands.
/// Three execs on a status change costs three times what one does, and this
/// runs on the draw thread.
pub fn publish_args(pane: &str, status: Option<Status>, counts: Counts) -> Vec<String> {
    let mut args = set_args(pane, STATUS_OPTION, status.map(word));
    args.push(";".to_owned());
    args.extend(set_args(pane, AGENTS_OPTION, (counts.held > 0).then(|| agents(counts)).as_deref()));
    args.push(";".to_owned());
    // Without this the bar would not repaint until status-interval, which is a
    // minute under a default tmuxbar -- long enough to look broken.
    args.extend(["refresh-client".to_owned(), "-S".to_owned()]);
    args
}

/// One beat, in the shape `publish_args` uses: the value and the repaint in a
/// single invocation.
///
/// tmuxbar cannot keep this time itself. Its status line repaints only when
/// something asks it to, and the terminal's own blink attribute is ignored by
/// ghostty -- so the pulse has to be drawn, and the thing that knows an agent is
/// waiting is the thing already talking to tmux.
pub fn pulse_args(lit: bool) -> Vec<String> {
    let mut args: Vec<String> = ["set-option", "-g", BLINK_OPTION, if lit { "1" } else { "0" }].iter().map(|arg| (*arg).to_owned()).collect();
    args.push(";".to_owned());
    args.extend(["refresh-client".to_owned(), "-S".to_owned()]);
    args
}

/// Setting one pane option, or clearing it when there is nothing to say.
fn set_args(pane: &str, option: &str, value: Option<&str>) -> Vec<String> {
    let mut args: Vec<String> = ["set-option", "-p"].iter().map(|arg| (*arg).to_owned()).collect();
    if value.is_none() {
        args.push("-u".to_owned());
    }
    args.extend(["-t".to_owned(), pane.to_owned(), option.to_owned()]);
    if let Some(value) = value {
        args.push(value.to_owned());
    }
    args
}

/// Say what this atrium needs. Never fails loudly -- an atrium that cannot
/// reach tmux is not a broken atrium.
pub fn publish(pane: &str, status: Option<Status>, counts: Counts) {
    run(publish_args(pane, status, counts));
}

/// Hand the bar the half of the beat it should be drawing.
pub fn pulse(lit: bool) {
    run(pulse_args(lit));
}

/// One tmux invocation, with nothing said about how it went: an atrium that
/// cannot reach tmux is not a broken atrium.
fn run(args: Vec<String>) {
    let _ = Command::new("tmux").args(args).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).status();
}

#[cfg(test)]
#[path = "../tests/core/tmux.rs"]
mod tests;
