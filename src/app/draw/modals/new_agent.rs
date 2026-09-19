use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState},
};

use crate::app::state::{layout, picker::Picker};

const WIDTH: u16 = 56;
const HEIGHT: u16 = 16;

/// The "hold something new" box: a filter, the CLI to launch, and the projects
/// still matching.
pub fn draw(frame: &mut Frame, full: Rect, picker: &Picker) {
    let area = layout::centered(WIDTH, HEIGHT, full);
    // Without this the stage shows through the gaps in the modal.
    frame.render_widget(Clear, area);

    let block = Block::default().borders(Borders::ALL).title(" new agent ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [header, list_area] = layout::stack_header(inner, 2);

    let header_lines = vec![
        Line::from(vec![Span::raw("> "), Span::raw(picker.filter()), Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))]),
        Line::from(vec![
            Span::styled("tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::raw(picker.kind()),
            Span::styled("   enter hold · esc cancel", Style::default().add_modifier(Modifier::DIM)),
        ]),
    ];
    frame.render_widget(ratatui::widgets::Paragraph::new(header_lines), header);

    if let Some(error) = picker.error() {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Line::from(Span::styled(format!("  {error}"), Style::default().add_modifier(Modifier::BOLD)))).wrap(ratatui::widgets::Wrap { trim: true }),
            list_area,
        );
        return;
    }

    let matches = picker.matches();
    if matches.is_empty() {
        frame.render_widget(ratatui::widgets::Paragraph::new(Line::from(Span::styled("  no project matches", Style::default().add_modifier(Modifier::DIM)))), list_area);
        return;
    }

    let items: Vec<ListItem> = matches.iter().map(|project| ListItem::new(Line::from(Span::raw(format!(" {}", project.name))))).collect();
    let list = List::new(items).highlight_style(Style::default().add_modifier(Modifier::REVERSED));

    let mut state = ListState::default();
    state.select(Some(picker.selected()));
    frame.render_stateful_widget(list, list_area, &mut state);
}

#[cfg(test)]
#[path = "../../../tests/app/draw/modals/new_agent.rs"]
mod tests;
