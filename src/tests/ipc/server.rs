use super::*;
use std::{
    io::Write,
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};

/// Delivery crosses a thread boundary, so the test waits for it rather than
/// assuming it has already happened.
fn next_reports(server: &StatusServer) -> Vec<Report> {
    let deadline = Instant::now() + Duration::from_secs(5);
    while Instant::now() < deadline {
        let reports = server.drain();
        if !reports.is_empty() {
            return reports;
        }
        thread::sleep(Duration::from_millis(10));
    }
    Vec::new()
}

#[test]
fn the_socket_exists_once_bound_and_is_gone_once_dropped() {
    let path = {
        let server = StatusServer::bind().expect("bind");
        let path = server.path().to_path_buf();
        assert!(path.exists(), "socket should exist while atrium is running");
        path
    };
    assert!(!path.exists(), "socket should not outlive the atrium that made it");
}

#[test]
fn a_line_written_to_the_socket_arrives_as_a_report() {
    let server = StatusServer::bind().expect("bind");

    let mut stream = UnixStream::connect(server.path()).expect("connect");
    writeln!(stream, "9\tUserPromptSubmit").expect("write");
    drop(stream);

    let reports = next_reports(&server);
    assert_eq!(reports, vec![Report { agent_id: 9, event: "UserPromptSubmit".to_owned() }]);
}

#[test]
fn several_hooks_can_report_at_once() {
    let server = StatusServer::bind().expect("bind");

    for id in 1..=3 {
        let mut stream = UnixStream::connect(server.path()).expect("connect");
        writeln!(stream, "{id}\tStop").expect("write");
    }

    let deadline = Instant::now() + Duration::from_secs(5);
    let mut seen = Vec::new();
    while seen.len() < 3 && Instant::now() < deadline {
        seen.extend(server.drain());
        thread::sleep(Duration::from_millis(10));
    }

    let mut ids: Vec<u64> = seen.iter().map(|r| r.agent_id).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec![1, 2, 3]);
}

#[test]
fn rubbish_on_the_socket_is_ignored_rather_than_crashing_the_server() {
    let server = StatusServer::bind().expect("bind");

    let mut stream = UnixStream::connect(server.path()).expect("connect");
    writeln!(stream, "garbage\n4\tStop").expect("write");
    drop(stream);

    assert_eq!(next_reports(&server), vec![Report { agent_id: 4, event: "Stop".to_owned() }]);
}

#[test]
fn two_servers_in_one_process_do_not_collide() {
    let a = StatusServer::bind().expect("bind a");
    let b = StatusServer::bind().expect("bind b");
    assert_ne!(a.path(), b.path());
    assert!(a.path().exists() && b.path().exists());
}

#[test]
fn binding_clears_sockets_left_by_an_atrium_that_never_exited() {
    // A pid far above pid_max can never be alive, so this stands in for the
    // socket a killed atrium leaves behind.
    let server = StatusServer::bind().expect("bind");
    let dir = server.path().parent().expect("socket dir").to_path_buf();
    let stale = dir.join("4294967294-0.sock");
    std::fs::write(&stale, b"").expect("plant a stale socket");
    assert!(stale.exists());

    let second = StatusServer::bind().expect("bind again");

    assert!(!stale.exists(), "a dead atrium's socket should have been swept");
    assert!(server.path().exists(), "a live atrium's socket must survive the sweep");
    assert!(second.path().exists());
}
