use super::*;

fn report(agent_id: u64, event: &str, at: Option<u64>) -> Report {
    Report { agent_id, event: event.to_owned(), at }
}

#[test]
fn a_report_survives_a_round_trip() {
    for at in [None, Some(1_790_532_030_147)] {
        let sent = report(42, "Stop", at);
        assert_eq!(Report::parse(&sent.encode()), Some(sent));
    }
}

#[test]
fn the_encoding_is_one_tab_separated_line() {
    assert_eq!(report(7, "Notification", None).encode(), "7\tNotification\n");
    assert_eq!(report(7, "Notification", Some(1234)).encode(), "7\tNotification\t1234\n");
}

#[test]
fn a_trailing_newline_is_optional() {
    assert_eq!(Report::parse("3\tStop"), Some(report(3, "Stop", None)));
    assert_eq!(Report::parse("3\tStop\r\n"), Some(report(3, "Stop", None)));
    assert_eq!(Report::parse("3\tStop\t99\r\n"), Some(report(3, "Stop", Some(99))));
}

#[test]
fn malformed_lines_are_rejected_rather_than_guessed_at() {
    assert!(Report::parse("").is_none());
    assert!(Report::parse("no-tab-here").is_none());
    assert!(Report::parse("notanumber\tStop").is_none());
    assert!(Report::parse("5\t").is_none(), "an empty event name says nothing");
    assert!(Report::parse("5\tStop\tsoon").is_none(), "a stamp that is there has to be a stamp");
    assert!(Report::parse("5\tStop\t1\textra").is_none());
}

#[test]
fn now_is_the_wall_clock_in_milliseconds() {
    // Late 2020s, give or take: a stamp in seconds or microseconds would be
    // three orders of magnitude out and compare wrongly against everything.
    let now = now_ms();
    assert!((1_600_000_000_000..10_000_000_000_000).contains(&now), "{now}");
}
