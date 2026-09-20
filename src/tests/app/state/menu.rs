use super::*;

fn keymap() -> Keymap {
    Keymap::default()
}

fn agent_menu() -> Menu {
    Menu::for_agent((10, 5), 2, "guitar", &keymap())
}

#[test]
fn a_menu_on_a_row_offers_that_agent_by_name() {
    let menu = agent_menu();
    let labels: Vec<&str> = menu.items.iter().map(|item| item.label.as_str()).collect();

    assert!(labels.iter().any(|label| label.contains("guitar")), "{labels:?}");
    assert!(menu.items.iter().any(|item| item.action == Action::Focus(2)));
    assert!(menu.items.iter().any(|item| item.action == Action::Dismiss(2)));
}

#[test]
fn a_menu_off_a_row_still_offers_what_is_always_possible() {
    let menu = Menu::general((0, 0), None, &keymap());
    let actions: Vec<Action> = menu.items.iter().map(|item| item.action).collect();

    for expected in [Action::New, Action::Sidebar, Action::Settings, Action::Quit] {
        assert!(actions.contains(&expected), "{expected:?} missing from {actions:?}");
    }
    assert!(!actions.iter().any(|action| matches!(action, Action::Focus(_))), "nothing was clicked, so there is nothing to go to");
}

#[test]
fn a_menu_off_a_row_can_still_close_the_agent_on_the_stage() {
    let menu = Menu::general((0, 0), Some((1, "atrium")), &keymap());

    assert!(menu.items.iter().any(|item| item.action == Action::Dismiss(1)));
}

#[test]
fn entries_carry_the_chord_that_does_the_same_thing() {
    let menu = Menu::general((0, 0), None, &keymap());
    let quit = menu.items.iter().find(|item| item.action == Action::Quit).expect("a quit entry");

    assert_eq!(quit.chord, keymap().gesture(keymap().quit), "the menu shows the whole gesture, not just the second key");
}

#[test]
fn the_cursor_starts_on_something_that_can_be_picked() {
    assert!(agent_menu().picked().is_some());
}

#[test]
fn moving_steps_over_a_separator_rather_than_stopping_on_one() {
    let mut menu = agent_menu();
    for _ in 0..menu.items.len() {
        menu.move_down();
        assert!(menu.picked().is_some(), "the cursor landed on a separator");
    }
}

#[test]
fn moving_wraps_in_both_directions() {
    let mut menu = agent_menu();
    let first = menu.selected;

    menu.move_up();
    assert_ne!(menu.selected, first, "up from the top should go somewhere");

    for _ in 0..menu.items.len() {
        menu.move_down();
    }
    assert!(menu.picked().is_some());
}

#[test]
fn the_box_is_wide_enough_for_its_widest_line() {
    let menu = agent_menu();
    let widest = menu.items.iter().map(|item| item.label.chars().count() + item.chord.chars().count()).max().unwrap_or(0) as u16;

    assert!(menu.width() > widest, "a label needs room for the border and padding around it");
    assert_eq!(menu.height() as usize, menu.items.len() + 2, "a border above and below");
}

#[test]
fn the_box_opens_where_it_was_clicked() {
    let menu = agent_menu();
    let area = menu.area(Rect::new(0, 0, 200, 60));

    assert_eq!((area.x, area.y), (10, 5));
}

#[test]
fn a_box_that_would_hang_off_an_edge_is_pulled_back_inside() {
    let full = Rect::new(0, 0, 60, 20);
    let menu = Menu::for_agent((59, 19), 0, "atrium", &keymap());
    let area = menu.area(full);

    assert!(area.x + area.width <= full.width, "it should not run off the right");
    assert!(area.y + area.height <= full.height, "it should not run off the bottom");
}

#[test]
fn a_box_too_big_for_the_frame_is_shrunk_to_fit() {
    let full = Rect::new(0, 0, 10, 4);
    let area = agent_menu().area(full);

    assert!(area.width <= full.width);
    assert!(area.height <= full.height);
}

#[test]
fn a_click_lands_on_the_entry_under_it() {
    let full = Rect::new(0, 0, 200, 60);
    let menu = agent_menu();
    let area = menu.area(full);

    assert_eq!(menu.item_at(full, area.x + 2, area.y), None, "the top border is not an entry");
    assert_eq!(menu.item_at(full, area.x + 2, area.y + 1), Some(0));
    assert_eq!(menu.item_at(full, area.x + 2, area.y + 2), Some(1));
}

#[test]
fn a_click_on_a_separator_lands_on_nothing() {
    let full = Rect::new(0, 0, 200, 60);
    let menu = agent_menu();
    let area = menu.area(full);
    let separator = menu.items.iter().position(|item| item.action == Action::Separator).expect("a separator");

    assert_eq!(menu.item_at(full, area.x + 2, area.y + 1 + separator as u16), None);
}

#[test]
fn a_click_outside_the_box_is_not_on_the_menu() {
    let full = Rect::new(0, 0, 200, 60);
    let menu = agent_menu();
    let area = menu.area(full);

    assert!(!menu.covers(full, area.x.saturating_sub(1), area.y));
    assert!(menu.covers(full, area.x, area.y), "the border still counts as the menu");
}

#[test]
fn selecting_a_separator_is_ignored() {
    let mut menu = agent_menu();
    let before = menu.selected;
    let separator = menu.items.iter().position(|item| item.action == Action::Separator).expect("a separator");

    menu.select(separator);

    assert_eq!(menu.selected, before, "the cursor should not have moved onto it");
}
