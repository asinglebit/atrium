use ratatui::{
    Frame,
    layout::Rect,
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Clear, Paragraph},
};

use crate::{
    app::{
        draw::pane,
        state::{
            layout,
            profile_editor::{Action, Editor, Step},
        },
    },
    helpers::{palette::Theme, text::truncate_with_ellipsis},
};

const WIDTH: u16 = 60;

/// Guitar's modal marker: the selected row is pointed at, the rest are not.
const SELECTED: &str = ">";
const UNSELECTED: &str = " ";

/// Editing a profile: an action list, a prompt, or a confirmation, whichever
/// step the editor is on.
pub fn draw(frame: &mut Frame, full: Rect, editor: &Editor, name: &str, theme: &Theme) {
    let (title, mut lines) = match &editor.step {
        Step::Actions { selected, .. } => (" profile ", actions(*selected, name, theme)),
        Step::ConfirmDelete { .. } => (" delete profile ", confirm(name, theme)),
        Step::Asking { prompt, input } => (" profile ", asking(prompt.title(), input, theme)),
    };

    // A refusal stays in the modal, so another answer can be given without
    // losing where you were.
    if let Some(error) = &editor.error {
        lines.insert(lines.len() - 1, Line::from(Span::styled(truncate_with_ellipsis(error, WIDTH as usize - 4), Style::default().fg(theme.COLOR_RED))));
        lines.insert(lines.len() - 1, Line::default());
    }

    let height = lines.len() as u16 + 2;
    let area = layout::centered(WIDTH, height, full);
    // Without this the settings view shows through the gaps.
    frame.render_widget(Clear, area);

    let block = pane::modal_block(theme, title);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    frame.render_widget(Paragraph::new(lines), inner);
}

fn actions(selected: usize, name: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = vec![Line::from(vec![Span::styled("profile  ", Style::default().fg(theme.COLOR_GREY_600)), Span::styled(name.to_owned(), Style::default().fg(theme.COLOR_TEXT))]), Line::default()];

    for (index, action) in Action::ALL.iter().enumerate() {
        let is_selected = index == selected;
        let colour = if matches!(action, Action::Delete) { theme.COLOR_RED } else { theme.COLOR_TEXT };
        let style = if is_selected { Style::default().fg(colour).bg(theme.background_or_default(theme.COLOR_GREY_800)) } else { Style::default().fg(colour) };
        lines.push(Line::from(Span::styled(format!("{} {}", if is_selected { SELECTED } else { UNSELECTED }, action.label()), style)));
    }

    lines.push(Line::default());
    lines.push(hint("enter picks · esc closes", theme));
    lines
}

fn confirm(name: &str, theme: &Theme) -> Vec<Line<'static>> {
    vec![
        Line::from(Span::styled("this cannot be undone", Style::default().fg(theme.COLOR_GREY_600))),
        Line::default(),
        Line::from(Span::styled(truncate_with_ellipsis(name, WIDTH as usize - 4), Style::default().fg(theme.COLOR_RED))),
        Line::default(),
        hint("enter deletes · esc cancels", theme),
    ]
}

fn asking(title: &str, input: &str, theme: &Theme) -> Vec<Line<'static>> {
    let mut lines = vec![
        Line::from(Span::styled(title.to_owned(), Style::default().fg(theme.COLOR_GREY_600))),
        Line::default(),
        Line::from(vec![
            Span::styled("> ", Style::default().fg(theme.COLOR_GREY_600)),
            Span::styled(truncate_with_ellipsis(input, WIDTH as usize - 6), Style::default().fg(theme.COLOR_TEXT)),
            Span::styled("_", Style::default().add_modifier(Modifier::SLOW_BLINK)),
        ]),
        Line::default(),
    ];

    lines.push(hint("enter confirms · esc cancels", theme));
    lines
}

fn hint(text: &str, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(text.to_owned(), Style::default().fg(theme.COLOR_GREY_700)))
}

#[cfg(test)]
#[path = "../../../tests/app/draw/modals/profile.rs"]
mod tests;
