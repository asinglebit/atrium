use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Clear, List, ListItem},
};

use crate::{
    app::{
        draw::pane,
        state::menu::{Action, Menu},
    },
    helpers::{palette::Theme, text::truncate},
};

/// What a separator is drawn with, matching the frame's own line.
const RULE: char = '─';

/// The menu floats over whatever was right-clicked, so it clears its own box
/// first -- otherwise the agent's screen shows through it.
pub fn draw(frame: &mut Frame, full: Rect, menu: &Menu, theme: &Theme) {
    let area = menu.area(full);
    if area.width < 4 || area.height < 3 {
        return;
    }

    frame.render_widget(Clear, area);

    let block = pane::modal_block(theme, "");
    let inner = block.inner(area);
    frame.render_widget(block, area);

    let items: Vec<ListItem> =
        menu.items.iter().enumerate().map(|(index, item)| ListItem::new(line(&item.label, &item.chord, item.action, index == menu.selected, inner.width as usize, theme))).collect();
    frame.render_widget(List::new(items), inner);
}

fn line(label: &str, chord: &str, action: Action, selected: bool, width: usize, theme: &Theme) -> Line<'static> {
    if action == Action::Separator {
        return Line::from(Span::styled(RULE.to_string().repeat(width), Style::default().fg(theme.COLOR_BORDER)));
    }

    let label = truncate(label, width.saturating_sub(chord.chars().count() + 2));
    let gap = width.saturating_sub(label.chars().count() + chord.chars().count() + 1);
    let background = if selected { Some(theme.background_or_default(theme.COLOR_GREY_800)) } else { None };
    let paint = |style: Style| match background {
        Some(background) => style.bg(background),
        None => style,
    };

    Line::from(vec![
        Span::styled(label, paint(Style::default().fg(theme.COLOR_GREY_300))),
        Span::styled(" ".repeat(gap), paint(Style::default())),
        Span::styled(format!("{chord} "), paint(Style::default().fg(theme.COLOR_GREY_600))),
    ])
}

#[cfg(test)]
#[path = "../../tests/app/draw/menu.rs"]
mod tests;
