use super::*;
use crate::{
    app::state::profile_editor::Prompt,
    core::profiles_file::{StoredProfile, StoredProfiles},
};
use ratatui::{Terminal, backend::TestBackend};

fn rendered(editor: &Editor, name: &str) -> String {
    let mut terminal = Terminal::new(TestBackend::new(70, 20)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), editor, name, &Theme::classic(), true)).expect("draw");
    terminal.backend().buffer().content().chunks(70).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

#[test]
fn the_action_list_names_the_profile_and_every_action() {
    let out = rendered(&Editor::manage(0), "work");

    assert!(out.contains("work"), "{out}");
    for action in Action::ALL {
        assert!(out.contains(action.label()), "{} missing:\n{out}", action.label());
    }
}

#[test]
fn the_selected_action_is_the_one_pointed_at() {
    let mut editor = Editor::manage(0);
    editor.move_down();

    let out = rendered(&editor, "work");
    let pointed = out.lines().find(|line| line.contains("> ")).expect("a marker");

    assert!(pointed.contains(Action::Rename.label()), "{out}");
}

#[test]
fn a_prompt_shows_what_it_wants_and_what_has_been_typed() {
    let mut editor = Editor::add();
    for c in "personal".chars() {
        editor.push(c);
    }

    let out = rendered(&editor, "");

    assert!(out.contains(Prompt::AddName.title()), "{out}");
    assert!(out.contains("personal"), "{out}");
}

#[test]
fn a_refusal_stays_in_the_modal() {
    let mut editor = Editor::add();
    editor.fail("there is already a profile called \"work\"");

    let out = rendered(&editor, "");

    assert!(out.contains("already a profile"), "{out}");
    assert!(out.contains("esc cancels"), "the hint should survive the error being inserted:\n{out}");
}

#[test]
fn deleting_says_it_cannot_be_undone() {
    let mut editor = Editor::manage(0);
    for _ in 0..4 {
        editor.move_down();
    }
    editor.confirm(&StoredProfiles { default: String::new(), profiles: vec![StoredProfile::named("work")] });

    let out = rendered(&editor, "work");

    assert!(out.contains("cannot be undone"), "{out}");
    assert!(out.contains("work"), "it should name what it is deleting:\n{out}");
}

#[test]
fn the_box_is_bordered_so_it_reads_as_floating() {
    assert!(rendered(&Editor::manage(0), "work").contains('╭'));
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    let mut terminal = Terminal::new(TestBackend::new(10, 4)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), &Editor::manage(0), "work", &Theme::classic(), true)).expect("should not panic on a small frame");
}
