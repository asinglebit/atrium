use super::*;
use std::{
    path::Path,
    time::{Duration, Instant},
};

/// The same liveness test the socket sweep uses, and the reason this one test
/// is Linux-only.
#[cfg(target_os = "linux")]
fn is_running(pid: u32) -> bool {
    Path::new(&format!("/proc/{pid}")).exists()
}

/// Spins until the agent's screen says what we are waiting for, or gives up.
fn wait_for(session: &PtySession, needle: &str) -> bool {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        if let Ok(parser) = session.parser().lock()
            && parser.screen().contents().contains(needle)
        {
            return true;
        }
        thread::sleep(Duration::from_millis(20));
    }
    false
}

#[test]
fn a_spawned_command_reaches_the_screen() {
    let mut cmd = CommandBuilder::new("sh");
    cmd.args(["-c", "printf atrium-lives"]);

    let session = PtySession::spawn(cmd, 24, 80).expect("pty should open");
    assert!(wait_for(&session, "atrium-lives"), "child output never reached the parser");
}

#[test]
fn input_written_to_the_pty_is_seen_by_the_child() {
    let mut cmd = CommandBuilder::new("sh");
    cmd.args(["-c", "read line; printf \"got-%s\" \"$line\""]);

    let mut session = PtySession::spawn(cmd, 24, 80).expect("pty should open");
    session.write(b"ping\r").expect("write should reach the pty");
    assert!(wait_for(&session, "got-ping"), "child never saw the input");
}

#[test]
fn resize_updates_the_parser_screen() {
    let cmd = CommandBuilder::new("cat");
    let mut session = PtySession::spawn(cmd, 24, 80).expect("pty should open");

    session.resize(40, 120).expect("resize should succeed");

    let parser = session.parser().lock().expect("parser lock");
    assert_eq!(parser.screen().size(), (40, 120));
}

#[test]
fn a_finished_child_stops_being_alive() {
    let mut cmd = CommandBuilder::new("sh");
    cmd.args(["-c", "exit 0"]);

    let mut session = PtySession::spawn(cmd, 24, 80).expect("pty should open");

    let deadline = Instant::now() + Duration::from_secs(5);
    while session.is_alive() && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(!session.is_alive(), "child should have been reaped");
}

#[test]
#[cfg(target_os = "linux")]
fn dropping_a_session_ends_its_child() {
    let mut cmd = CommandBuilder::new("sleep");
    cmd.args(["60"]);

    let session = PtySession::spawn(cmd, 24, 80).expect("pty should open");
    let pid = session.pid().expect("a spawned child has a pid");
    assert!(is_running(pid), "the child should have started");

    drop(session);

    let deadline = Instant::now() + Duration::from_secs(5);
    while is_running(pid) && Instant::now() < deadline {
        thread::sleep(Duration::from_millis(20));
    }
    assert!(!is_running(pid), "the child outlived the session that held it");
}

/// What the screen is showing right now, which is the live screen or the
/// history, depending on where it has been scrolled.
fn showing(session: &PtySession) -> String {
    session.parser().lock().expect("parser lock").screen().contents()
}

/// A screen ten rows tall with more than ten rows written to it, so the first
/// line is only reachable through the history.
fn overflowed() -> PtySession {
    let mut cmd = CommandBuilder::new("sh");
    cmd.args(["-c", "echo top-marker; for i in $(seq 1 60); do echo filler; done; echo bottom-marker"]);

    let session = PtySession::spawn(cmd, 10, 40).expect("pty should open");
    assert!(wait_for(&session, "bottom-marker"), "the child's last line never reached the parser");
    session
}

#[test]
fn the_history_above_the_screen_can_be_scrolled_back_to() {
    let session = overflowed();
    assert!(!showing(&session).contains("top-marker"), "the first line should have scrolled off a ten-row screen");

    session.scroll_back(60);

    assert!(showing(&session).contains("top-marker"), "scrolling back should reach what the screen has passed");
}

#[test]
fn scrolling_forward_again_ends_on_the_live_screen() {
    let session = overflowed();
    session.scroll_back(60);

    session.scroll_forward(60);

    assert!(showing(&session).contains("bottom-marker"), "{}", showing(&session));
    assert!(!showing(&session).contains("top-marker"), "forward should have come all the way back");
}

#[test]
fn typing_is_what_show_live_is_for() {
    let session = overflowed();
    session.scroll_back(60);

    session.show_live();

    assert!(showing(&session).contains("bottom-marker"), "the live screen is where a keypress puts you:\n{}", showing(&session));
}

#[test]
fn scrolling_stops_at_both_ends_rather_than_running_off() {
    let session = overflowed();

    session.scroll_forward(10_000);
    assert!(showing(&session).contains("bottom-marker"), "there is nothing below the live screen to reach");

    session.scroll_back(10_000);
    assert!(showing(&session).contains("top-marker"), "there is nothing above the oldest line it kept");
}

#[test]
fn a_cli_that_never_asked_for_the_mouse_is_not_reported_to() {
    let mut cmd = CommandBuilder::new("sh");
    cmd.args(["-c", "printf asked-for-nothing"]);

    let session = PtySession::spawn(cmd, 10, 40).expect("pty should open");
    assert!(wait_for(&session, "asked-for-nothing"), "child output never reached the parser");

    assert!(!session.wants_mouse(), "nothing here turned mouse reporting on");
}

#[test]
fn a_cli_that_asked_for_the_mouse_is() {
    let mut cmd = CommandBuilder::new("sh");
    cmd.args(["-c", "printf '\\033[?1000hasked-for-the-mouse'"]);

    let session = PtySession::spawn(cmd, 10, 40).expect("pty should open");
    assert!(wait_for(&session, "asked-for-the-mouse"), "child output never reached the parser");

    assert!(session.wants_mouse(), "the child turned mouse reporting on and should be sent reports");
}
