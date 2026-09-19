use super::*;
use std::time::{Duration, Instant};

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
