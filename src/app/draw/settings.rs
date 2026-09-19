use ratatui::{
    Frame,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::Paragraph,
};

use crate::{
    app::{
        input::keymap::Keymap,
        state::settings::{Settings, Tab},
    },
    helpers::{
        logo,
        palette::{THEME_PRESETS, Theme},
        text::truncate,
        version::VERSION,
    },
};

/// Fills the gap between a label and its value so the eye can follow one across.
const FILL: char = '·';

/// What the view says about working itself, under the tab bar.
const HINT: &str = "tab switches · enter picks · esc closes";

/// Margin kept either side of the centred column, as guitar keeps for its own.
const MARGIN: usize = 8;

/// However wide the frame gets, the column stops growing here -- a settings row
/// stretched across a full-screen terminal is unreadable.
const MAX_CONTENT_WIDTH: usize = 48;

/// The settings view, in guitar's shape: one centred column, a logo above it
/// where guitar puts its heatmap, then the tab bar and the section below.
pub fn draw(frame: &mut Frame, area: Rect, settings: &mut Settings, keymap: &Keymap, theme: &Theme) {
    let width = content_width(area);
    let body = body(settings, keymap, theme, width);

    settings.row_lines = body.rows;
    settings.tab_line = body.tab_line;
    settings.trap_scroll(body.lines.len(), area.height as usize);

    // The paragraph carries the background itself, so the lines it does not
    // reach are painted rather than left as the terminal's own.
    let shown: Vec<Line> = body.lines.into_iter().skip(settings.scroll).take(area.height as usize).collect();
    frame.render_widget(Paragraph::new(shown).style(theme.background_style()), area);
}

/// Everything the view draws, and where the parts a click can land on ended up.
/// Built in one pass so drawing, scrolling and clicking cannot disagree about
/// which line is which.
struct Body {
    lines: Vec<Line<'static>>,
    /// The line each selectable row sits on, in order.
    rows: Vec<usize>,
    tab_line: usize,
}

fn content_width(area: Rect) -> usize {
    (area.width as usize).saturating_sub(MARGIN).min(MAX_CONTENT_WIDTH)
}

fn body(settings: &Settings, keymap: &Keymap, theme: &Theme, width: usize) -> Body {
    let mut lines: Vec<Line<'static>> = vec![Line::default()];

    let rows = logo::rows_for(width);
    for (index, row) in rows.iter().enumerate() {
        // The brighter purple on top and the deeper one below, which is how
        // guitar splits its own logo across two greens.
        let colour = if index < logo::BRIGHT_ROWS || rows.len() == 1 { theme.COLOR_PURPLE } else { theme.COLOR_DURPLE };
        lines.push(Line::from(Span::styled(*row, Style::default().fg(colour))).centered());
    }

    lines.push(Line::default());
    lines.push(filled("version", VERSION, width, theme, None));
    lines.push(Line::default());
    let tab_line = lines.len();
    lines.push(tab_bar(settings, theme, width));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(HINT, Style::default().fg(theme.COLOR_GREY_700))).centered());

    let (heading, entries) = match settings.tab() {
        Tab::Shortcuts => ("keys", shortcut_entries(keymap)),
        Tab::Themes => ("themes", theme_entries(theme)),
    };

    lines.push(Line::default());
    lines.push(section(heading, width, theme));
    lines.push(Line::default());

    let mut row_lines = Vec::with_capacity(entries.len());
    for (index, (label, value)) in entries.iter().enumerate() {
        let background = if index == settings.selected() {
            Some(theme.background_or_default(theme.COLOR_GREY_800))
        } else if index.is_multiple_of(2) {
            Some(theme.background_or_default(theme.COLOR_GREY_900))
        } else {
            None
        };
        row_lines.push(lines.len());
        lines.push(filled(label, value, width, theme, background));
    }

    Body { lines, rows: row_lines, tab_line }
}

/// A centred heading over a section, left-aligned inside the column the way
/// guitar's are.
fn section(label: &str, width: usize, theme: &Theme) -> Line<'static> {
    let padded = format!(" {label}{}", " ".repeat(width.saturating_sub(label.chars().count() + 1)));
    Line::from(Span::styled(padded, Style::default().fg(theme.COLOR_HIGHLIGHTED))).centered()
}

/// `label ········ value`, sized to the column and carrying the row's own
/// background so a stripe runs the whole way across it.
fn filled(label: &str, value: &str, width: usize, theme: &Theme, background: Option<Color>) -> Line<'static> {
    let label = truncate(label, width.saturating_sub(4));
    let value = truncate(value, width.saturating_sub(label.chars().count() + 3));
    let gap = width.saturating_sub(label.chars().count() + value.chars().count() + 3);

    let paint = |colour: Color| {
        let style = Style::default().fg(colour);
        match background {
            Some(background) => style.bg(background),
            None => style,
        }
    };

    Line::from(vec![
        Span::styled(format!(" {label} "), paint(theme.COLOR_GREY_300)),
        Span::styled(FILL.to_string().repeat(gap), paint(theme.COLOR_BORDER)),
        Span::styled(format!(" {value} "), paint(theme.COLOR_TEXT)),
    ])
    .centered()
}

/// How wide the tab bar's own content is: the column, unless the labels need
/// more than the column has. Both drawing and hit-testing centre on this.
fn tab_bar_width(width: usize) -> usize {
    let labels: usize = Tab::ALL.iter().map(|tab| tab.label().chars().count() + 3).sum();
    (labels + 1).max(width)
}

fn tab_bar(settings: &Settings, theme: &Theme, width: usize) -> Line<'static> {
    let mut spans = vec![Span::raw(" ")];
    let mut used = 1;
    for tab in Tab::ALL {
        let style = if tab == settings.tab() { Style::default().fg(theme.COLOR_GREY_200).bg(theme.background_or_default(theme.COLOR_GREY_800)) } else { Style::default().fg(theme.COLOR_GREY_600) };
        let label = format!(" {} ", tab.label());
        used += label.chars().count() + 1;
        spans.push(Span::styled(label, style));
        spans.push(Span::raw(" "));
    }
    // Padded out to the column so the bar centres with everything else rather
    // than around its own middle.
    spans.push(Span::raw(" ".repeat(tab_bar_width(width).saturating_sub(used))));

    Line::from(spans).centered()
}

fn shortcut_entries(keymap: &Keymap) -> Vec<(String, String)> {
    keymap.actions().iter().map(|(name, chord)| ((*name).to_owned(), chord.label())).collect()
}

fn theme_entries(theme: &Theme) -> Vec<(String, String)> {
    THEME_PRESETS.iter().map(|preset| (preset.label.to_owned(), if preset.theme.name == theme.name { "in use".to_owned() } else { String::new() })).collect()
}

/// Which row is under a click, worked out from where the last draw put them.
pub fn row_at(settings: &Settings, area: Rect, row: u16) -> Option<usize> {
    let line = settings.scroll + usize::from(row.checked_sub(area.y)?);
    settings.row_lines.iter().position(|at| *at == line)
}

/// Which tab a click on the bar lands on, if any. The bar is centred inside the
/// column, so the walk has to start where the centring put it.
pub fn tab_at(settings: &Settings, area: Rect, column: u16, row: u16) -> Option<Tab> {
    if settings.scroll + usize::from(row.checked_sub(area.y)?) != settings.tab_line {
        return None;
    }

    let width = tab_bar_width(content_width(area)) as u16;
    let mut x = area.x + (area.width.saturating_sub(width)) / 2 + 1;
    for tab in Tab::ALL {
        let label = tab.label().chars().count() as u16 + 2;
        if column >= x && column < x + label {
            return Some(tab);
        }
        x += label + 1;
    }
    None
}

#[cfg(test)]
#[path = "../../tests/app/draw/settings.rs"]
mod tests;
