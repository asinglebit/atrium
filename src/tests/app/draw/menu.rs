use super::*;
use crate::app::input::keymap::Keymap;
use ratatui::{Terminal, backend::TestBackend};

fn rendered(menu: &Menu, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).expect("test terminal");
    terminal.draw(|frame| draw(frame, frame.area(), menu, &Theme::classic())).expect("draw");
    terminal.backend().buffer().content().chunks(width as usize).map(|row| row.iter().map(|cell| cell.symbol()).collect::<String>()).collect::<Vec<_>>().join("\n")
}

fn menu() -> Menu {
    Menu::for_agent((2, 1), 0, "guitar", &Keymap::default())
}

#[test]
fn every_entry_is_drawn_with_its_chord() {
    let out = rendered(&menu(), 60, 20);

    assert!(out.contains("close guitar"), "{out}");
    assert!(out.contains("new agent"), "{out}");
    assert!(out.contains(&Keymap::default().quit.label()), "the chord belongs beside the entry:\n{out}");
}

#[test]
fn a_separator_is_drawn_as_a_line() {
    assert!(rendered(&menu(), 60, 20).contains("──"));
}

#[test]
fn the_box_is_bordered_so_it_reads_as_floating() {
    let out = rendered(&menu(), 60, 20);

    assert!(out.contains('╭'), "a rounded corner, like the modals:\n{out}");
}

#[test]
fn what_is_behind_the_menu_does_not_show_through_it() {
    let menu = menu();
    let mut terminal = Terminal::new(TestBackend::new(60, 20)).expect("test terminal");
    terminal
        .draw(|frame| {
            let filler = ratatui::widgets::Paragraph::new("xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
            frame.render_widget(filler, frame.area());
            draw(frame, frame.area(), &menu, &Theme::classic());
        })
        .expect("draw");

    let area = menu.area(ratatui::layout::Rect::new(0, 0, 60, 20));
    let buffer = terminal.backend().buffer();
    let inside: String = (area.y + 1..area.y + area.height - 1).map(|row| buffer[(area.x + 1, row)].symbol().to_owned()).collect();

    assert!(!inside.contains('x'), "the menu should have cleared what it covers, got {inside:?}");
}

#[test]
fn it_fits_a_terminal_too_small_for_it() {
    rendered(&menu(), 8, 4);
    rendered(&menu(), 2, 2);
}
