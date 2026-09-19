use ratatui::{
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Padding},
};

use crate::{core::agent::Status, helpers::palette::Theme};

/// A pane, the way guitar draws one outside zen mode: no border at all, just
/// padding and the themed background. The zebra striping below is what makes
/// the column visible, which is why no line is needed to separate it.
pub fn block<'a>(theme: &Theme, title: impl Into<Line<'a>>) -> Block<'a> {
    Block::default().padding(Padding { left: 1, right: 1, top: 0, bottom: 0 }).title(title).title_style(Style::default().fg(theme.COLOR_GREY_600)).style(theme.background_style())
}

/// The modal floats over the stage, so unlike a pane it does need an edge.
/// Rounded, because that is the corner guitar's zen border uses.
pub fn modal_block<'a>(theme: &Theme, title: impl Into<Line<'a>>) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme.COLOR_GREY_700))
        .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
        .title(title)
        .title_style(Style::default().fg(theme.COLOR_GREY_600))
        .style(theme.background_style())
}

/// Lifted from guitar's `pane_window::zebra_list_items` so rows stripe the same
/// way: the selected row takes GREY_800, and every other row GREY_900.
pub fn zebra_list_items<'a>(lines: Vec<Line<'a>>, visible_height: usize, selected: usize, is_focused: bool, theme: &Theme) -> Vec<ListItem<'a>> {
    (0..visible_height)
        .map(|index| {
            let line = lines.get(index).cloned().unwrap_or_default();
            let is_selected = is_focused && index == selected;

            let mut item = if is_selected {
                let spans: Vec<Span> = line.iter().map(|span| Span::styled(span.content.clone(), span.style)).collect();
                ListItem::new(Line::from(spans)).style(Style::default().bg(theme.background_or_default(theme.COLOR_GREY_800)))
            } else {
                ListItem::new(line)
            };

            if !is_selected && index.is_multiple_of(2) {
                item = item.style(Style::default().bg(theme.background_or_default(theme.COLOR_GREY_900)));
            }

            item
        })
        .collect()
}

/// What each status is painted in, drawn from guitar's palette rather than
/// colours of atrium's own, so the two tools never disagree about what red is.
pub fn status_color(theme: &Theme, status: Status) -> Color {
    match status {
        Status::Idle => theme.COLOR_GREY_400,
        Status::Working => theme.COLOR_AMBER,
        Status::NeedsInput => theme.COLOR_GREEN,
        Status::Error => theme.COLOR_RED,
        Status::Exited => theme.COLOR_GREY_600,
    }
}
