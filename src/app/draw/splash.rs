use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem},
};

use crate::{
    app::state::splash::Splash,
    core::profile::Profile,
    helpers::{logo, palette::Theme, text::truncate_with_ellipsis},
};

/// What guitar wraps its selected splash row in. Brackets rather than a
/// background, so a centred row keeps its width.
const SELECTED_LEFT: &str = "⏵ ";
const SELECTED_RIGHT: &str = " ⏴";

/// What the list is. atrium holds agents, so what it offers is what it can
/// launch one as.
const HEADING: &str = "harnesses";

const HINT: &str = "actions: enter holds one here | ctrl+t pick a project | ctrl+s settings";

/// Everything above the list: the wordmark, a blank, the heading, a blank, the
/// hint, a blank, and the failure line when there is one.
fn chrome_rows(area: Rect, splash: &Splash) -> usize {
    logo::splash_rows_for(area.width as usize).len() + 5 + if splash.error.is_some() { 2 } else { 0 }
}

/// Blank rows above the content, so the whole thing sits in the middle of the
/// frame the way guitar's does.
fn padding(area: Rect, splash: &Splash, count: usize) -> usize {
    (area.height as usize).saturating_sub(chrome_rows(area, splash) + count) / 2
}

/// The row the list starts on, which is what a click is measured against.
pub fn first_row(area: Rect, splash: &Splash, count: usize) -> u16 {
    area.y + (padding(area, splash, count) + chrome_rows(area, splash)) as u16
}

/// What atrium shows when it is holding nothing: the wordmark, and what it
/// could hold. Guitar's splash, with the recent repositories replaced by the
/// profiles -- see `Profiles` in the docs.
pub fn draw(frame: &mut Frame, area: Rect, splash: &Splash, profiles: &[Profile], theme: &Theme) {
    let rows = logo::splash_rows_for(area.width as usize);
    let mut lines: Vec<Line> = (0..padding(area, splash, profiles.len())).map(|_| Line::default()).collect();

    for (index, row) in rows.iter().enumerate() {
        let colour = logo::tone(index, rows, theme);
        lines.push(Line::from(Span::styled(*row, Style::default().fg(colour))).centered());
    }

    lines.push(Line::default());
    lines.push(Line::from(Span::styled(HEADING, Style::default().fg(theme.COLOR_TEXT))).centered());
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(truncate_with_ellipsis(HINT, area.width as usize), Style::default().fg(theme.COLOR_GREY_600))).centered());
    lines.push(Line::default());

    // A failed launch stays on the splash so another choice can be made.
    if let Some(error) = &splash.error {
        lines.push(Line::from(Span::styled(truncate_with_ellipsis(error, area.width as usize), Style::default().fg(theme.COLOR_RED))).centered());
        lines.push(Line::default());
    }

    for (index, profile) in profiles.iter().enumerate() {
        let is_selected = index == splash.selected();
        let colour = if is_selected { theme.COLOR_GRASS } else { theme.COLOR_TEXT };
        let label = Span::styled(truncate_with_ellipsis(&profile.label(), area.width as usize), Style::default().fg(colour));

        // Brackets rather than a highlight, so the row keeps its width and the
        // list does not shift under the cursor.
        let line = if is_selected {
            Line::from(vec![Span::styled(SELECTED_LEFT, Style::default().fg(theme.COLOR_GRASS)), label, Span::styled(SELECTED_RIGHT, Style::default().fg(theme.COLOR_GRASS))])
        } else {
            Line::from(label)
        };
        lines.push(line.centered());
    }

    frame.render_widget(List::new(lines.into_iter().map(ListItem::from).collect::<Vec<_>>()).style(theme.background_style()), area);
}

#[cfg(test)]
#[path = "../../tests/app/draw/splash.rs"]
mod tests;
