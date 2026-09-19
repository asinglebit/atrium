use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, List, Padding, Paragraph},
};

use crate::{
    app::{
        draw::pane,
        input::keymap::Keymap,
        state::{
            layout,
            settings::{Settings, Tab},
        },
    },
    helpers::{
        palette::{THEME_PRESETS, Theme},
        text::truncate,
    },
};

/// Fills the gap between a label and its value, the way guitar's settings rows
/// do, so the eye can follow one across.
const FILL: char = '·';

/// How many rows the tab header costs, so the caller can scroll by the same
/// number of rows this actually shows.
pub const HEADER_HEIGHT: u16 = 2;

/// The settings view. Takes the whole inside of the frame, like guitar's.
pub fn draw(frame: &mut Frame, area: Rect, settings: &Settings, keymap: &Keymap, theme: &Theme) {
    let [tabs, rest] = layout::stack_header(area, HEADER_HEIGHT);
    draw_tabs(frame, tabs, settings, theme);

    let split = rest.width.saturating_sub(1);
    let body = Rect { width: split, ..rest };
    let gutter = Rect { x: rest.x + split, width: 1, ..rest };

    // No title here: the tab row above already says which section this is.
    let block = Block::default().padding(Padding { left: 1, right: 1, top: 0, bottom: 0 }).style(theme.background_style());
    let inner = block.inner(body);
    frame.render_widget(block, body);

    let lines = match settings.tab() {
        Tab::Shortcuts => shortcut_lines(keymap, theme, inner.width as usize),
        Tab::Themes => theme_lines(theme, inner.width as usize),
    };

    let visible = inner.height as usize;
    let shown: Vec<_> = lines.into_iter().skip(settings.scroll).collect();
    let selected = settings.selected().saturating_sub(settings.scroll);
    frame.render_widget(List::new(pane::zebra_list_items(shown, visible, selected, true, theme)), inner);

    pane::draw_gutter(frame, gutter, theme, settings.len(), visible, settings.scroll);
}

fn draw_tabs(frame: &mut Frame, area: Rect, settings: &Settings, theme: &Theme) {
    let mut spans = vec![Span::raw(" ")];
    for tab in Tab::ALL {
        let style = if tab == settings.tab() { Style::default().fg(theme.COLOR_GREY_200).bg(theme.background_or_default(theme.COLOR_GREY_800)) } else { Style::default().fg(theme.COLOR_GREY_600) };
        spans.push(Span::styled(format!(" {} ", tab.label()), style));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::styled("  tab switches · enter picks · esc closes", Style::default().fg(theme.COLOR_GREY_700)));
    frame.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// `label ········ value`, sized to the pane.
fn filled_line(label: &str, value: &str, width: usize, theme: &Theme) -> Line<'static> {
    let label = truncate(label, width.saturating_sub(4));
    let value = truncate(value, width.saturating_sub(label.chars().count() + 3));
    let gap = width.saturating_sub(label.chars().count() + value.chars().count() + 3);

    Line::from(vec![
        Span::styled(format!(" {label} "), Style::default().fg(theme.COLOR_GREY_300)),
        Span::styled(FILL.to_string().repeat(gap), Style::default().fg(theme.COLOR_BORDER)),
        Span::styled(format!(" {value} "), Style::default().fg(theme.COLOR_TEXT)),
    ])
}

fn shortcut_lines(keymap: &Keymap, theme: &Theme, width: usize) -> Vec<Line<'static>> {
    keymap.actions().iter().map(|(name, chord)| filled_line(name, &chord.label(), width, theme)).collect()
}

fn theme_lines(theme: &Theme, width: usize) -> Vec<Line<'static>> {
    THEME_PRESETS
        .iter()
        .map(|preset| {
            let in_use = preset.theme.name == theme.name;
            filled_line(preset.label, if in_use { "in use" } else { "" }, width, theme)
        })
        .collect()
}

/// Which row is under a click, counting from below the tab header.
pub fn row_at(area: Rect, offset: usize, row: u16) -> Option<usize> {
    let first = area.y.checked_add(HEADER_HEIGHT)?;
    (row >= first && row < area.y.saturating_add(area.height)).then(|| offset + usize::from(row - first))
}

/// Which tab a click on the header lands on, if any. The row is `" [ label ] "`
/// laid out left to right, so the widths have to be walked the same way.
pub fn tab_at(area: Rect, column: u16, row: u16) -> Option<Tab> {
    if row != area.y {
        return None;
    }
    let mut x = area.x + 1;
    for tab in Tab::ALL {
        let width = tab.label().chars().count() as u16 + 2;
        if column >= x && column < x + width {
            return Some(tab);
        }
        x += width + 1;
    }
    None
}

#[cfg(test)]
#[path = "../../tests/app/draw/settings.rs"]
mod tests;
