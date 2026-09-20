use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Borders, ListItem, Padding, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    core::{agent::Status, registry::Registry},
    helpers::{palette::Theme, scroll, text::truncate},
};

/// The scrollbar's own column doubles as the line separating the panes, which
/// is how guitar draws one: the track is the border, the thumb rides on it.
const TRACK: &str = "│";
const THUMB: &str = "▌";

/// A pane, the way guitar draws one outside zen mode: no border at all, just
/// padding and the themed background. The zebra striping below is what makes
/// the column visible, which is why no line is needed to separate it.
///
/// Untitled on purpose. A title costs the top row, and the status line already
/// counts what is held.
pub fn block(theme: &Theme) -> Block<'static> {
    Block::default().padding(Padding { left: 1, right: 1, top: 0, bottom: 0 }).style(theme.background_style())
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

/// The column between two panes: a scrollbar when there is something to
/// scroll, and the plain border line when there is not.
///
/// ratatui draws nothing at all for a scrollbar with no content length, so the
/// separator has to be drawn by hand in that case or the panes run together.
pub fn draw_gutter(frame: &mut Frame, area: Rect, theme: &Theme, total: usize, visible: usize, offset: usize) {
    let length = scroll::content_length(total, visible);
    if length == 0 {
        frame.render_widget(Block::default().borders(Borders::LEFT).border_style(Style::default().fg(theme.COLOR_BORDER)), area);
        return;
    }

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(None)
        .end_symbol(None)
        .track_symbol(Some(TRACK))
        .thumb_symbol(THUMB)
        .track_style(Style::default().fg(theme.COLOR_BORDER))
        .thumb_style(Style::default().fg(theme.COLOR_GREY_600));

    frame.render_stateful_widget(scrollbar, area, &mut ScrollbarState::new(length).position(offset));
}

/// How each status is drawn. The colours are drawn from guitar's palette rather
/// than colours of atrium's own, so the two tools never disagree about what red
/// is, and tmuxbar paints a window by the same rule -- blue for a question,
/// orange for a wait, green for a finished agent, red for a failed one.
///
/// The two statuses that are still waiting pulse: on the dark half of the beat
/// they drop to grey, which is `lit` being false. The other three are settled --
/// nothing about them will change on its own -- so they ignore it and hold
/// their colour.
///
/// The pulse is drawn rather than asked for. `Modifier::SLOW_BLINK` emits SGR 5,
/// which ghostty parses and ignores, so anything that has to blink here has to
/// blink by being drawn two ways.
pub fn status_style(theme: &Theme, status: Status, lit: bool) -> Style {
    let (color, pulses) = match status {
        Status::Idle => (theme.COLOR_GREEN, false),
        Status::Working => (theme.COLOR_ORANGE, true),
        Status::NeedsInput => (theme.COLOR_BLUE, true),
        Status::Error => (theme.COLOR_RED, false),
        Status::Exited => (theme.COLOR_GREY_600, false),
    };
    Style::default().fg(if pulses && !lit { theme.COLOR_GREY_600 } else { color })
}

/// The `_` a text field is waiting behind, on the lit half of the pulse and the
/// dark one. A space rather than nothing, so the line does not shift under it.
pub fn cursor(lit: bool) -> &'static str {
    if lit { "_" } else { " " }
}

/// However long the branches are, a name never gets squeezed below this.
const MIN_NAME_WIDTH: usize = 8;

/// One line per held agent: status mark, jump number, name, and branch. Shared
/// so the sidebar and the goto list cannot drift apart.
pub fn agent_lines<'a>(registry: &Registry, theme: &Theme, spinner: char, lit: bool, width: usize) -> Vec<Line<'a>> {
    let branches: Vec<String> = registry.agents().iter().map(|agent| agent.git().map(|git| format!("{}{}", git.branch, if git.dirty { "*" } else { "" })).unwrap_or_default()).collect();
    let profiles: Vec<String> = registry.agents().iter().map(|agent| agent.profile.clone().unwrap_or_default()).collect();

    let widest = |values: &[String]| values.iter().map(|value| value.chars().count()).max().unwrap_or(0);
    // A column nothing fills costs nothing, separating space included.
    let trailing = |column: usize| if column == 0 { 0 } else { column + 1 };

    // One column each for every row, so the names line up on the left and the
    // rest line up on the right instead of each row finding its own edge.
    let marker = 4;
    let room = width.saturating_sub(marker);
    // The branch gives ground before the profile does: two rows on one project
    // are told apart by the profile and nothing else.
    let profile_width = widest(&profiles).min(room.saturating_sub(MIN_NAME_WIDTH + 1));
    let branch_width = widest(&branches).min(room.saturating_sub(MIN_NAME_WIDTH + 1 + trailing(profile_width)));
    let name_width = room.saturating_sub(trailing(profile_width) + trailing(branch_width));

    registry
        .agents()
        .iter()
        .zip(&branches)
        .zip(&profiles)
        .enumerate()
        .map(|(index, ((agent, branch), profile))| {
            // Ten rows can be reached by a key, numbered 1 to 9 and then 0.
            // Past that there is no key to write down, so nothing is written.
            let key = match index {
                0..=8 => format!("{} ", index + 1),
                9 => "0 ".to_owned(),
                _ => "  ".to_owned(),
            };
            // A working agent spins where the others show a steady glyph.
            let mark = if agent.status == Status::Working { spinner.to_string() } else { agent.status.glyph().to_owned() };
            let body = if agent.has_exited() { theme.COLOR_GREY_600 } else { theme.COLOR_GREY_300 };

            let mut spans = vec![
                Span::styled(format!("{mark} "), status_style(theme, agent.status, lit)),
                Span::styled(key, Style::default().fg(theme.COLOR_GREY_600)),
                Span::styled(format!("{:<name_width$}", truncate(&agent.name, name_width)), Style::default().fg(body)),
            ];
            if profile_width > 0 {
                spans.push(Span::styled(format!(" {:<profile_width$}", truncate(profile, profile_width)), Style::default().fg(theme.COLOR_GREY_600)));
            }
            if branch_width > 0 {
                spans.push(Span::styled(format!(" {:>branch_width$}", truncate(branch, branch_width)), Style::default().fg(theme.COLOR_GREY_600)));
            }
            Line::from(spans)
        })
        .collect()
}
