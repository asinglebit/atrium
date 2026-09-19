use super::*;
use crate::{core::projects::Project, helpers::palette::Theme};
use ratatui::{Terminal, backend::TestBackend};
use std::path::PathBuf;

fn picker_of(names: &[&str]) -> Picker {
    Picker::new(names.iter().map(|name| Project { name: (*name).to_owned(), path: PathBuf::from("/p").join(name) }).collect())
}

fn rendered(picker: &Picker) -> String {
    let mut terminal = Terminal::new(TestBackend::new(70, 20)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), picker, &Theme::default())).expect("draw");
    terminal.backend().buffer().content().chunks(70).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

#[test]
fn it_lists_the_projects_it_could_hold() {
    let out = rendered(&picker_of(&["atrium", "guitar"]));
    assert!(out.contains("new agent"), "{out}");
    assert!(out.contains("atrium") && out.contains("guitar"), "{out}");
}

#[test]
fn it_shows_which_cli_enter_would_launch() {
    assert!(rendered(&picker_of(&["atrium"])).contains("claude"));
}

#[test]
fn it_echoes_what_has_been_typed() {
    let mut picker = picker_of(&["atrium", "guitar"]);
    picker.push('g');
    let out = rendered(&picker);
    assert!(out.contains("> g"), "{out}");
    assert!(!out.contains("atrium"), "a filtered-out project should be gone:\n{out}");
}

#[test]
fn it_says_so_when_nothing_matches() {
    let mut picker = picker_of(&["atrium"]);
    for c in "zzz".chars() {
        picker.push(c);
    }
    assert!(rendered(&picker).contains("no project matches"));
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    let mut terminal = Terminal::new(TestBackend::new(20, 6)).expect("test terminal");
    let picker = picker_of(&["atrium"]);
    terminal.draw(|frame| draw(frame, frame.area(), &picker, &Theme::default())).expect("should not panic on a small frame");
}

#[test]
fn a_failed_launch_is_shown_instead_of_the_list() {
    let mut picker = picker_of(&["atrium", "guitar"]);
    picker.set_error("Unable to spawn codex");

    let out = rendered(&picker);
    assert!(out.contains("Unable to spawn codex"), "{out}");
    assert!(!out.contains("guitar"), "the list should give way to the error:\n{out}");
}
