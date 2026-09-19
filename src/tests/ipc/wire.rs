use super::*;

#[test]
fn a_report_survives_a_round_trip() {
    let report = Report { agent_id: 42, event: "Stop".to_owned() };
    assert_eq!(Report::parse(&report.encode()), Some(report));
}

#[test]
fn the_encoding_is_one_tab_separated_line() {
    assert_eq!(Report { agent_id: 7, event: "Notification".to_owned() }.encode(), "7\tNotification\n");
}

#[test]
fn a_trailing_newline_is_optional() {
    assert_eq!(Report::parse("3\tStop"), Some(Report { agent_id: 3, event: "Stop".to_owned() }));
    assert_eq!(Report::parse("3\tStop\r\n"), Some(Report { agent_id: 3, event: "Stop".to_owned() }));
}

#[test]
fn malformed_lines_are_rejected_rather_than_guessed_at() {
    assert!(Report::parse("").is_none());
    assert!(Report::parse("no-tab-here").is_none());
    assert!(Report::parse("notanumber\tStop").is_none());
    assert!(Report::parse("5\t").is_none(), "an empty event name says nothing");
}
