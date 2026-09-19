use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, List},
};

use crate::{
    app::{
        draw::pane,
        state::{layout, picker::Picker},
    },
    helpers::palette::Theme,
};

const WIDTH: u16 = 56;
const HEIGHT: u16 = 16;

/// The "hold something new" box: a filter, the CLI to launch, and the projects
/// still matching.
pub fn draw(frame: &mut Frame, full: Rect, picker: &Picker, theme: &Theme) {
    let area = layout::centered(WIDTH, HEIGHT, full);
    // Without this the stage shows through the gaps in the modal.
    frame.render_widget(Clear, area);

    let block = pane::modal_block(theme, " new agent ");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [header, list_area] = layout::stack_header(inner, 2);

    let header_lines = vec![
        Line::from(vec![Span::raw("> "), Span::raw(picker.filter()), Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK))]),
        Line::from(vec![
            Span::styled("tab", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw(" "),
            Span::styled(picker.kind(), Style::default().fg(theme.COLOR_GREY_300)),
            Span::styled("   enter hold · esc cancel", Style::default().fg(theme.COLOR_GREY_600)),
        ]),
    ];
    frame.render_widget(ratatui::widgets::Paragraph::new(header_lines), header);

    if let Some(error) = picker.error() {
        frame.render_widget(
            ratatui::widgets::Paragraph::new(Line::from(Span::styled(format!("  {error}"), Style::default().fg(theme.COLOR_RED)))).wrap(ratatui::widgets::Wrap { trim: true }),
            list_area,
        );
        return;
    }

    let matches = picker.matches();
    if matches.is_empty() {
        frame.render_widget(ratatui::widgets::Paragraph::new(Line::from(Span::styled("  no project matches", Style::default().fg(theme.COLOR_GREY_600)))), list_area);
        return;
    }

    let lines: Vec<Line> = matches.iter().map(|project| Line::from(Span::styled(project.name.clone(), Style::default().fg(theme.COLOR_GREY_300)))).collect();
    let items = pane::zebra_list_items(lines, list_area.height as usize, picker.selected(), true, theme);
    frame.render_widget(List::new(items), list_area);
}

#[cfg(test)]
#[path = "../../../tests/app/draw/modals/new_agent.rs"]
mod tests;
