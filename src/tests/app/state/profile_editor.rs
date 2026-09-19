use super::*;

fn both() -> StoredProfiles {
    StoredProfiles { default: "work".to_owned(), profiles: vec![StoredProfile::named("work"), StoredProfile::named("personal")] }
}

/// Runs a committing outcome against a set, the way the app does.
fn apply(outcome: Outcome, profiles: &mut StoredProfiles) -> Result<(), String> {
    match outcome {
        Outcome::Commit(change) => change(profiles),
        _ => panic!("expected the step to have finished"),
    }
}

fn typed(editor: &mut Editor, text: &str) {
    for c in text.chars() {
        editor.push(c);
    }
}

fn input_of(editor: &Editor) -> String {
    match &editor.step {
        Step::Asking { input, .. } => input.clone(),
        other => panic!("expected a prompt, got {other:?}"),
    }
}

#[test]
fn adding_asks_for_a_name_first() {
    let editor = Editor::add();

    assert!(matches!(editor.step, Step::Asking { prompt: Prompt::AddName, .. }));
    assert_eq!(editor.index(), None, "there is no profile yet to be working on");
}

#[test]
fn adding_guesses_the_directory_from_the_name() {
    let mut editor = Editor::add();
    typed(&mut editor, "personal");

    editor.confirm(&StoredProfiles::default());

    assert_eq!(input_of(&editor), "~/.claude-personal", "the convention is prefilled so enter is enough");
}

#[test]
fn adding_chains_name_then_directory_then_args() {
    let mut profiles = StoredProfiles::default();
    let mut editor = Editor::add();

    typed(&mut editor, "work");
    editor.confirm(&profiles);
    editor.confirm(&profiles);
    typed(&mut editor, "--append-system-prompt-file /etc/p.md");
    apply(editor.confirm(&profiles), &mut profiles).expect("commit");

    let added = &profiles.profiles[0];
    assert_eq!(added.name, "work");
    assert_eq!(added.config_dir, "~/.claude-work", "what was typed is kept raw");
    assert_eq!(added.args, ["--append-system-prompt-file", "/etc/p.md"]);
    assert_eq!(profiles.default, "work", "the first one added becomes the default");
}

#[test]
fn an_empty_name_is_refused_without_moving_on() {
    let mut editor = Editor::add();

    editor.confirm(&StoredProfiles::default());

    assert!(editor.error.is_some(), "it should say why");
    assert!(matches!(editor.step, Step::Asking { prompt: Prompt::AddName, .. }), "and stay where it was");
}

#[test]
fn typing_clears_a_refusal() {
    let mut editor = Editor::add();
    editor.confirm(&StoredProfiles::default());

    editor.push('w');

    assert!(editor.error.is_none());
}

#[test]
fn a_name_already_taken_is_refused_when_it_is_applied() {
    let mut profiles = both();
    let mut editor = Editor::add();

    typed(&mut editor, "work");
    editor.confirm(&profiles);
    editor.confirm(&profiles);

    assert!(apply(editor.confirm(&profiles), &mut profiles).is_err());
    assert_eq!(profiles.profiles.len(), 2, "nothing should have been appended");
}

#[test]
fn managing_opens_on_the_first_action() {
    let editor = Editor::manage(1);

    assert_eq!(editor.step, Step::Actions { index: 1, selected: 0 });
    assert_eq!(editor.index(), Some(1));
}

#[test]
fn the_action_list_wraps_in_both_directions() {
    let mut editor = Editor::manage(0);

    editor.move_up();
    assert_eq!(editor.step, Step::Actions { index: 0, selected: Action::ALL.len() - 1 });

    editor.move_down();
    assert_eq!(editor.step, Step::Actions { index: 0, selected: 0 });
}

#[test]
fn setting_the_default_moves_it() {
    let mut profiles = both();
    let mut editor = Editor::manage(1);

    apply(editor.confirm(&profiles), &mut profiles).expect("commit");

    assert_eq!(profiles.default, "personal");
}

#[test]
fn renaming_prefills_with_the_name_it_has() {
    let profiles = both();
    let mut editor = Editor::manage(0);
    editor.move_down();

    editor.confirm(&profiles);

    assert_eq!(input_of(&editor), "work", "editing should be a correction, not retyping");
}

#[test]
fn renaming_applies_to_the_profile_it_was_opened_on() {
    let mut profiles = both();
    let mut editor = Editor::manage(0);
    editor.move_down();
    editor.confirm(&profiles);

    typed(&mut editor, "s");
    apply(editor.confirm(&profiles), &mut profiles).expect("commit");

    assert_eq!(profiles.profiles[0].name, "works");
    assert_eq!(profiles.default, "works", "the default follows a rename");
}

#[test]
fn editing_the_directory_prefills_and_keeps_what_is_typed_raw() {
    let mut profiles = both();
    let mut editor = Editor::manage(0);
    editor.move_down();
    editor.move_down();
    editor.confirm(&profiles);

    assert_eq!(input_of(&editor), "~/.claude-work");
    typed(&mut editor, "-2");
    apply(editor.confirm(&profiles), &mut profiles).expect("commit");

    assert_eq!(profiles.profiles[0].config_dir, "~/.claude-work-2");
}

#[test]
fn editing_args_prefills_with_them_joined_and_splits_them_back() {
    let mut profiles = StoredProfiles {
        default: String::new(),
        profiles: vec![StoredProfile { name: "work".to_owned(), program: String::new(), config_dir: String::new(), args: vec!["--a".to_owned(), "--b".to_owned()], env: Vec::new() }],
    };
    let mut editor = Editor::manage(0);
    for _ in 0..3 {
        editor.move_down();
    }
    editor.confirm(&profiles);

    assert_eq!(input_of(&editor), "--a --b");
    typed(&mut editor, "   --c");
    apply(editor.confirm(&profiles), &mut profiles).expect("commit");

    assert_eq!(profiles.profiles[0].args, ["--a", "--b", "--c"], "runs of spaces should not become empty arguments");
}

#[test]
fn deleting_asks_first() {
    let profiles = both();
    let mut editor = Editor::manage(0);
    for _ in 0..4 {
        editor.move_down();
    }

    let outcome = editor.confirm(&profiles);

    assert!(matches!(outcome, Outcome::Continue), "it should not have deleted anything yet");
    assert_eq!(editor.step, Step::ConfirmDelete { index: 0 });
}

#[test]
fn confirming_a_delete_removes_it_and_moves_the_default() {
    let mut profiles = both();
    let mut editor = Editor::manage(0);
    for _ in 0..4 {
        editor.move_down();
    }
    editor.confirm(&profiles);

    apply(editor.confirm(&profiles), &mut profiles).expect("commit");

    assert_eq!(profiles.profiles.len(), 1);
    assert_eq!(profiles.default, "personal");
}

#[test]
fn esc_backs_out_of_the_whole_thing() {
    let mut editor = Editor::add();
    typed(&mut editor, "half");
    editor.confirm(&StoredProfiles::default());

    assert!(matches!(editor.cancel(), Outcome::Close), "half an added profile is not worth keeping");
}

#[test]
fn every_prompt_says_what_it_wants() {
    for prompt in [
        Prompt::AddName,
        Prompt::AddConfigDir { name: "w".to_owned() },
        Prompt::AddArgs { name: "w".to_owned(), config_dir: "d".to_owned() },
        Prompt::Rename(0),
        Prompt::EditConfigDir(0),
        Prompt::EditArgs(0),
    ] {
        assert!(!prompt.title().is_empty());
    }
    assert!(Prompt::AddArgs { name: String::new(), config_dir: String::new() }.title().contains("split"), "the lossy bit has to be said out loud");
}

#[test]
fn every_action_is_named() {
    for action in Action::ALL {
        assert!(!action.label().is_empty());
    }
}
