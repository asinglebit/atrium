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

/// How the last turn to end here ended -- `done` or `error` -- while nobody was
/// looking. Set only at the moment it happens and never again, so the hook that
/// clears it when a window is visited has the last word until the next one.
pub const UNSEEN_OPTION: &str = "@atrium_unseen";

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
        Status::Done => "done",
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

/// Marks the window as holding a result nobody has seen, unless somebody is
/// looking at it right now -- a turn that ends in front of you is seen.
///
/// Worded as tmux's own `if-shell -F`, so the check and the write are one step
/// inside tmux rather than a question and an answer across two invocations.
/// A finish never covers a failure that is already waiting to be seen.
pub fn unseen_args(pane: &str, ended: Status) -> Vec<String> {
    let unwatched = "#{==:#{window_active_clients},0}";
    let (condition, value) = match ended {
        Status::Error => (unwatched.to_owned(), "error"),
        _ => (format!("#{{&&:{unwatched},#{{!=:#{{{UNSEEN_OPTION}}},error}}}}"), "done"),
    };
    vec!["if-shell".to_owned(), "-F".to_owned(), "-t".to_owned(), pane.to_owned(), condition, format!("set-option -p -t {pane} {UNSEEN_OPTION} {value}")]
}

/// Everything this atrium has said, unsaid, and the repaint that shows it.
pub fn clear_args(pane: &str) -> Vec<String> {
    let mut args = publish_args(pane, None, Counts::default());
    let repaint = args.split_off(args.len() - 2);
    args.extend(set_args(pane, UNSEEN_OPTION, None));
    args.push(";".to_owned());
    args.extend(repaint);
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

/// What this atrium needs, what just ended, or both, as one invocation.
pub fn announce_args(pane: &str, levels: Option<(Option<Status>, Counts)>, ended: Option<Status>) -> Vec<String> {
    let mut args = match levels {
        Some((status, counts)) => publish_args(pane, status, counts),
        None => vec!["refresh-client".to_owned(), "-S".to_owned()],
    };
    if let Some(ended) = ended {
        // Before the repaint, so the repaint shows it.
        let repaint = args.split_off(args.len() - 2);
        args.extend(unseen_args(pane, ended));
        args.push(";".to_owned());
        args.extend(repaint);
    }
    args
}

/// Say it. Never fails loudly -- an atrium that cannot reach tmux is not a
/// broken atrium.
pub fn publish(pane: &str, levels: Option<(Option<Status>, Counts)>, ended: Option<Status>) {
    run(announce_args(pane, levels, ended));
}

/// Unsay everything, on the way out.
pub fn clear(pane: &str) {
    run(clear_args(pane));
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
