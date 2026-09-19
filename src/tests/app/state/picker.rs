use super::*;
use std::path::PathBuf;

fn picker_of(names: &[&str]) -> Picker {
    Picker::new(names.iter().map(|name| Project { name: (*name).to_owned(), path: PathBuf::from("/projects").join(name) }).collect())
}

fn names(picker: &Picker) -> Vec<String> {
    picker.matches().into_iter().map(|project| project.name.clone()).collect()
}

#[test]
fn an_empty_filter_offers_everything() {
    let picker = picker_of(&["atrium", "guitar", "bazzite"]);
    assert_eq!(names(&picker).len(), 3);
}

#[test]
fn the_tightest_match_is_offered_first() {
    let mut picker = picker_of(&["asinglebit.github.io", "guitar"]);
    for c in "gui".chars() {
        picker.push(c);
    }
    // Both contain g, u and i in order; only one contains them together.
    assert_eq!(names(&picker), vec!["guitar", "asinglebit.github.io"]);
}

#[test]
fn an_exact_prefix_beats_a_scattered_match() {
    let mut picker = picker_of(&["documents", "dotfiles"]);
    for c in "dot".chars() {
        picker.push(c);
    }
    assert_eq!(names(&picker)[0], "dotfiles");
}

#[test]
fn an_empty_filter_leaves_the_list_alphabetical() {
    let picker = picker_of(&["guitar", "atrium", "bazzite"]);
    assert_eq!(names(&picker), vec!["atrium", "bazzite", "guitar"]);
}

#[test]
fn typing_narrows_to_a_substring() {
    let mut picker = picker_of(&["atrium", "guitar", "bazzite"]);
    for c in "gui".chars() {
        picker.push(c);
    }
    assert_eq!(names(&picker), vec!["guitar"]);
}

#[test]
fn matching_is_loose_so_initials_are_enough() {
    let mut picker = picker_of(&["payperpaper", "guitar"]);
    for c in "ppp".chars() {
        picker.push(c);
    }
    assert_eq!(names(&picker), vec!["payperpaper"]);
}

#[test]
fn matching_ignores_case() {
    let mut picker = picker_of(&["Bazzite"]);
    for c in "bz".chars() {
        picker.push(c);
    }
    assert_eq!(names(&picker), vec!["Bazzite"]);
}

#[test]
fn a_filter_that_matches_nothing_leaves_nothing_selected() {
    let mut picker = picker_of(&["atrium"]);
    for c in "zzz".chars() {
        picker.push(c);
    }
    assert!(picker.matches().is_empty());
    assert!(picker.selected_project().is_none());
}

#[test]
fn backspace_widens_the_list_again() {
    let mut picker = picker_of(&["atrium", "guitar"]);
    for c in "gui".chars() {
        picker.push(c);
    }
    assert_eq!(names(&picker).len(), 1);

    for _ in 0..3 {
        picker.backspace();
    }
    assert_eq!(names(&picker).len(), 2);
}

#[test]
fn typing_puts_the_selection_back_at_the_top() {
    let mut picker = picker_of(&["atrium", "guitar", "bazzite"]);
    picker.move_down();
    assert_eq!(picker.selected(), 1);

    picker.push('a');
    assert_eq!(picker.selected(), 0, "the list changed underneath, so the cursor should reset");
}

#[test]
fn the_selection_wraps_rather_than_sticking() {
    let mut picker = picker_of(&["a", "b"]);
    picker.move_up();
    assert_eq!(picker.selected(), 1);
    picker.move_down();
    assert_eq!(picker.selected(), 0);
}

#[test]
fn moving_in_an_empty_list_is_harmless() {
    let mut picker = picker_of(&[]);
    picker.move_down();
    picker.move_up();
    assert_eq!(picker.selected(), 0);
}

#[test]
fn the_selected_project_is_the_highlighted_one() {
    let mut picker = picker_of(&["atrium", "guitar"]);
    picker.move_down();
    assert_eq!(picker.selected_project().map(|p| p.name.as_str()), Some("guitar"));
}

#[test]
fn tab_cycles_through_every_agent_cli_and_comes_back() {
    let mut picker = picker_of(&["atrium"]);
    let first = picker.kind();
    let seen: Vec<&str> = (0..KINDS.len())
        .map(|_| {
            let kind = picker.kind();
            picker.cycle_kind();
            kind
        })
        .collect();

    assert_eq!(seen.len(), KINDS.len());
    assert_eq!(picker.kind(), first, "cycling all the way round should return to the start");
}

#[test]
fn claude_is_what_it_offers_first() {
    assert_eq!(picker_of(&[]).kind(), "claude");
}

#[test]
fn a_failed_launch_is_remembered() {
    let mut picker = picker_of(&["atrium"]);
    assert!(picker.error().is_none());

    picker.set_error("no such program");
    assert_eq!(picker.error(), Some("no such program"));
}

#[test]
fn changing_the_choice_clears_a_stale_error() {
    let mut picker = picker_of(&["atrium"]);

    picker.set_error("boom");
    picker.cycle_kind();
    assert!(picker.error().is_none(), "picking a different cli should drop the old failure");

    picker.set_error("boom");
    picker.push('a');
    assert!(picker.error().is_none(), "typing should drop the old failure");

    picker.set_error("boom");
    picker.backspace();
    assert!(picker.error().is_none());
}
