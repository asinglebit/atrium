use super::*;

/// A stand-in environment, so nothing here touches the real one -- setting a
/// variable is process-wide, and the suite runs its tests in parallel.
fn env(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> + use<> {
    let pairs: Vec<(String, String)> = pairs.iter().map(|(key, value)| ((*key).to_owned(), (*value).to_owned())).collect();
    move |name| pairs.iter().find(|(key, _)| key == name).map(|(_, value)| value.clone())
}

fn home() -> impl Fn(&str) -> Option<String> + use<> {
    env(&[("HOME", "/home/x")])
}

#[test]
fn a_bare_cli_is_a_profile_that_adds_nothing() {
    let profile = Profile::bare("claude");

    assert_eq!(profile.program, "claude");
    assert!(profile.args.is_empty());
    assert!(profile.env.is_empty());
}

#[test]
fn a_bare_cli_names_itself_once() {
    assert_eq!(Profile::bare("claude").label(), "claude");
}

#[test]
fn a_named_profile_says_which_cli_it_launches() {
    let profile = Profile { name: "work".into(), program: "claude".into(), args: Vec::new(), env: Vec::new() };

    assert_eq!(profile.label(), "claude · work");
}

#[test]
fn only_a_named_profile_earns_a_row_tag() {
    let named = Profile { name: "work".into(), program: "claude".into(), args: Vec::new(), env: Vec::new() };

    assert_eq!(named.tag().as_deref(), Some("work"));
    assert_eq!(Profile::bare("claude").tag(), None, "every row would carry the same tag, which says nothing");
}

#[test]
fn the_defaults_are_the_clis_atrium_knows() {
    let names: Vec<String> = defaults().iter().map(|profile| profile.name.clone()).collect();

    assert_eq!(names, DEFAULT_PROGRAMS);
}

#[test]
fn a_config_dir_is_found_by_the_variable_it_sets() {
    let profile = Profile { name: "work".into(), program: "claude".into(), args: Vec::new(), env: vec![(CONFIG_DIR_ENV.to_owned(), "/home/x/.claude-work".to_owned())] };

    assert_eq!(profile.config_dir(), Some("/home/x/.claude-work"));
    assert_eq!(Profile::bare("claude").config_dir(), None);
}

#[test]
fn a_leading_tilde_becomes_home() {
    assert_eq!(expand_with("~/.claude-work", home()), "/home/x/.claude-work");
    assert_eq!(expand_with("~", home()), "/home/x");
}

#[test]
fn a_tilde_that_is_not_a_home_directory_is_left_alone() {
    assert_eq!(expand_with("~work/thing", home()), "~work/thing", "only ~ and ~/ are a home directory");
    assert_eq!(expand_with("/opt/~/thing", home()), "/opt/~/thing", "a tilde in the middle is a tilde");
}

#[test]
fn a_variable_is_expanded_wherever_it_appears() {
    let dotfiles = || env(&[("DOTFILES", "/home/x/dotfiles")]);

    assert_eq!(expand_with("$DOTFILES/prompts/system.md", dotfiles()), "/home/x/dotfiles/prompts/system.md");
    assert_eq!(expand_with("before/$DOTFILES/after", dotfiles()), "before//home/x/dotfiles/after");
    assert_eq!(expand_with("${DOTFILES}x", dotfiles()), "/home/x/dotfilesx", "braces are what let a name end mid-word");
}

#[test]
fn an_unset_variable_expands_to_nothing_the_way_a_shell_would() {
    assert_eq!(expand_with("$NOTHING/x", env(&[])), "/x");
}

#[test]
fn a_dollar_that_names_nothing_stays_as_it_was_written() {
    assert_eq!(expand_with("costs $5", env(&[])), "costs $5");
    assert_eq!(expand_with("100$", env(&[])), "100$");
    assert_eq!(expand_with("${}", env(&[])), "${}");
    assert_eq!(expand_with("${5}", env(&[])), "${5}");
}

#[test]
fn an_unclosed_brace_does_not_swallow_the_rest() {
    assert_eq!(expand_with("${DOTFILES/prompts", env(&[("DOTFILES", "/home/x/dotfiles")])), "${DOTFILES/prompts");
}

#[test]
fn a_tilde_and_a_variable_can_appear_together() {
    let both = env(&[("HOME", "/home/x"), ("SUB", "work")]);

    assert_eq!(expand_with("~/.claude-$SUB", both), "/home/x/.claude-work");
}
