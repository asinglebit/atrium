use super::*;

fn counts(held: usize, working: usize, needs_input: usize, error: usize) -> Counts {
    Counts { held, working, needs_input, error }
}

#[test]
fn outside_tmux_there_is_no_pane() {
    assert!(pane_from(None, Some(OsString::from("%3"))).is_none(), "a pane without a server is a stale variable, not a pane");
    assert!(pane_from(Some(OsString::from("/tmp/s,1,0")), None).is_none());
    assert!(pane_from(None, None).is_none());
}

#[test]
fn an_empty_variable_is_the_same_as_an_unset_one() {
    assert!(pane_from(Some(OsString::from("")), Some(OsString::from("%3"))).is_none());
    assert!(pane_from(Some(OsString::from("/tmp/s,1,0")), Some(OsString::from(""))).is_none());
}

#[test]
fn inside_tmux_the_pane_is_reported() {
    assert_eq!(pane_from(Some(OsString::from("/tmp/s,1,0")), Some(OsString::from("%3"))), Some("%3".to_owned()));
}

#[test]
fn needs_input_is_one_word_so_a_format_string_can_match_it() {
    assert_eq!(word(Status::NeedsInput), "needs-input");
    assert!(!word(Status::NeedsInput).contains(' '), "a space would not survive a tmux format match");
    for status in [Status::Idle, Status::Working, Status::Done, Status::Error, Status::Exited] {
        assert!(!word(status).contains(' '));
    }
}

#[test]
fn a_finished_turn_has_a_word_of_its_own() {
    assert_eq!(word(Status::Done), "done");
    assert_ne!(word(Status::Done), word(Status::Idle), "finished and fresh are two different things to a window");
}

#[test]
fn the_counts_are_written_in_the_order_the_bar_reads_them() {
    assert_eq!(agents(counts(4, 2, 1, 0)), "4 2 1 0");
}

#[test]
fn publishing_sets_both_options_and_asks_for_a_repaint() {
    let args = publish_args("%3", Some(Status::NeedsInput), counts(2, 0, 1, 0));

    assert!(args.windows(3).any(|w| w == ["-t", "%3", STATUS_OPTION]));
    assert!(args.contains(&"needs-input".to_owned()));
    assert!(args.contains(&"2 0 1 0".to_owned()));
    assert_eq!(&args[args.len() - 2..], ["refresh-client", "-S"]);
}

#[test]
fn holding_nothing_clears_the_options_rather_than_saying_idle() {
    let args = publish_args("%3", None, counts(0, 0, 0, 0));

    assert_eq!(args.iter().filter(|arg| *arg == "-u").count(), 2, "both options are cleared");
    assert!(!args.iter().any(|arg| arg == "idle"), "a pane with no agents says nothing, rather than saying idle");
}

#[test]
fn the_commands_are_separated_the_way_tmux_separates_them() {
    let args = publish_args("%3", Some(Status::Idle), counts(1, 0, 0, 0));
    assert_eq!(args.iter().filter(|arg| *arg == ";").count(), 2, "three commands need two separators");
}

#[test]
fn a_beat_sets_the_option_globally_and_repaints() {
    let args = pulse_args(true);

    assert_eq!(args[..4], ["set-option", "-g", BLINK_OPTION, "1"]);
    assert!(args.contains(&";".to_owned()), "the repaint rides in the same invocation");
    assert!(args.ends_with(&["refresh-client".to_owned(), "-S".to_owned()]), "without this the window would not repaint until status-interval");
}

#[test]
fn the_dark_half_is_the_only_thing_that_says_zero() {
    // tmuxbar dims on an explicit 0 and lights on anything else, unset
    // included, so a stopped pulse can never leave a window dimmed.
    assert!(pulse_args(false).contains(&"0".to_owned()));
    assert!(!pulse_args(true).contains(&"0".to_owned()));
}

#[test]
fn an_ending_is_marked_only_where_nobody_is_looking() {
    let args = unseen_args("%3", Status::Done);

    assert_eq!(args[..4], ["if-shell", "-F", "-t", "%3"], "the check runs inside tmux, against this pane's window");
    assert!(args[4].contains("#{==:#{window_active_clients},0}"), "a turn that ends in front of you is seen: {}", args[4]);
    assert_eq!(args[5], format!("set-option -p -t %3 {UNSEEN_OPTION} done"));
}

#[test]
fn a_finish_never_covers_a_failure_still_waiting_to_be_seen() {
    let done = unseen_args("%3", Status::Done);
    assert!(done[4].contains(&format!("#{{!=:#{{{UNSEEN_OPTION}}},error}}")), "{}", done[4]);

    let error = unseen_args("%3", Status::Error);
    assert_eq!(error[4], "#{==:#{window_active_clients},0}", "a failure is marked over anything");
    assert!(error[5].ends_with(" error"));
}

#[test]
fn the_ending_rides_in_the_same_invocation_as_the_repaint() {
    let args = announce_args("%3", Some((Some(Status::Working), counts(2, 1, 0, 0))), Some(Status::Done));

    assert!(args.windows(3).any(|w| w == ["-t", "%3", STATUS_OPTION]), "the status still goes with it");
    let marker = args.iter().position(|arg| arg == "if-shell").expect("marker");
    let refresh = args.iter().position(|arg| arg == "refresh-client").expect("repaint");
    assert!(marker < refresh, "before the repaint, so the repaint shows it");
    assert_eq!(args.iter().filter(|arg| *arg == ";").count(), 3);
}

#[test]
fn an_ending_alone_is_said_without_the_status() {
    // One agent finishing beside another still working changes nothing else.
    let args = announce_args("%3", None, Some(Status::Done));

    assert_eq!(args[0], "if-shell");
    assert!(!args.contains(&STATUS_OPTION.to_owned()));
    assert_eq!(&args[args.len() - 2..], ["refresh-client", "-S"]);
}

#[test]
fn going_away_unsays_everything_including_the_marker() {
    let args = clear_args("%3");

    assert_eq!(args.iter().filter(|arg| *arg == "-u").count(), 3, "status, agents and the unseen marker: {args:?}");
    assert!(args.contains(&UNSEEN_OPTION.to_owned()));
    assert_eq!(&args[args.len() - 2..], ["refresh-client", "-S"]);
}
