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
    for status in [Status::Idle, Status::Working, Status::Error, Status::Exited] {
        assert!(!word(status).contains(' '));
    }
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
