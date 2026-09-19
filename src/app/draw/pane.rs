use ratatui::{
    style::{Color, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Padding},
};

use crate::{
    core::{agent::Status, registry::Registry},
    helpers::{palette::Theme, text::truncate},
};

/// A pane, the way guitar draws one outside zen mode: no border at all, just
/// padding and the themed background. The zebra striping below is what makes
/// the column visible, which is why no line is needed to separate it.
pub fn block<'a>(theme: &Theme, title: impl Into<Line<'a>>) -> Block<'a> {
    Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(theme.COLOR_BORDER))
        .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
        .title(title)
        .title_style(Style::default().fg(theme.COLOR_GREY_600))
        .style(theme.background_style())
}

/// The modal floats over the stage, so unlike a pane it does need an edge.
/// Rounded, because that is the corner guitar's zen border uses.
pub fn modal_block<'a>(theme: &Theme, title: impl Into<Line<'a>>) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .border_set(border::ROUNDED)
        .border_style(Style::default().fg(theme.COLOR_BORDER))
        .padding(Padding { left: 1, right: 1, top: 0, bottom: 0 })
        .title(title)
        .title_style(Style::default().fg(theme.COLOR_GREY_600))
        .style(theme.background_style())
}

/// The frame around the whole app, exactly as guitar draws it.
pub fn app_frame(theme: &Theme) -> Block<'static> {
    Block::default().borders(Borders::ALL).border_set(border::ROUNDED).border_style(Style::default().fg(theme.COLOR_BORDER))
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

/// However long the branches are, a name never gets squeezed below this.
const MIN_NAME_WIDTH: usize = 8;

/// One line per held agent: status mark, jump number, name, and branch. Shared
/// so the sidebar and the goto list cannot drift apart.
pub fn agent_lines<'a>(registry: &Registry, theme: &Theme, spinner: char, width: usize) -> Vec<Line<'a>> {
    let branches: Vec<String> = registry.agents().iter().map(|agent| agent.git().map(|git| format!("{}{}", git.branch, if git.dirty { "*" } else { "" })).unwrap_or_default()).collect();

    // One branch column for every row, so the names line up on the left and the
    // branches line up on the right instead of each row finding its own edge.
    let marker = 4;
    let room = width.saturating_sub(marker);
    let branch_width = branches.iter().map(|branch| branch.chars().count()).max().unwrap_or(0).min(room.saturating_sub(MIN_NAME_WIDTH + 1));
    let name_width = room.saturating_sub(if branch_width == 0 { 0 } else { branch_width + 1 });

    registry
        .agents()
        .iter()
        .zip(&branches)
        .enumerate()
        .map(|(index, (agent, branch))| {
            // Only the first nine get a jump number, so only they are numbered.
            let key = if index < 9 { format!("{} ", index + 1) } else { "  ".to_owned() };
            // A working agent spins where the others show a steady glyph.
            let mark = if agent.status == Status::Working { spinner.to_string() } else { agent.status.glyph().to_owned() };
            let body = if agent.has_exited() { theme.COLOR_GREY_600 } else { theme.COLOR_GREY_300 };

            let mut spans = vec![
                Span::styled(format!("{mark} "), Style::default().fg(status_color(theme, agent.status))),
                Span::styled(key, Style::default().fg(theme.COLOR_GREY_600)),
                Span::styled(format!("{:<name_width$}", truncate(&agent.name, name_width)), Style::default().fg(body)),
            ];
            if branch_width > 0 {
                spans.push(Span::styled(format!(" {:>branch_width$}", truncate(branch, branch_width)), Style::default().fg(theme.COLOR_GREY_600)));
            }
            Line::from(spans)
        })
        .collect()
}
