use std::path::Path;

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{List, ListItem, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    adapters::opencode,
    app::{
        input::keymap::Keymap,
        state::settings::{Selection, SelectionKind, Settings, Tab, TabHitbox},
    },
    core::{
        config::Config,
        installed::Found,
        layout_config,
        profile::KNOWN_PROGRAMS,
        profiles_file::{self, StoredProfiles},
        projects,
    },
    helpers::{
        logo,
        palette::{self, THEME_PRESETS, Theme},
        scroll,
        text::{fill_width, truncate_with_ellipsis},
        version::VERSION,
    },
};

/// Margin kept either side of the centred column, as guitar keeps for its own.
const MARGIN: usize = 8;

/// Guitar's column tops out at 53 heatmap weeks of two columns each. atrium has
/// no heatmap but keeps the ceiling: a settings row stretched across a
/// full-screen terminal is unreadable either way.
const MAX_CONTENT_WIDTH: usize = 106;

/// guitar's default symbol theme, hardcoded because atrium has no symbols.json.
const RADIO_ON: &str = "🞊";
const RADIO_OFF: &str = "🞅";
const COMPACT_TAB: &str = "•";

/// Two spaces between tab labels, as guitar spaces its own.
const TAB_GAP: &str = "  ";

/// Everything the view reads that is not its own state.
pub struct Context<'a> {
    pub keymap: &'a Keymap,
    pub theme: &'a Theme,
    pub profiles: &'a StoredProfiles,
    pub installed: &'a [Found],
    pub default_profile: usize,
    pub socket: &'a Path,
}

/// The settings view. `area` is the pane it fills; `border` is what the
/// scrollbar rides on, which is the app's own frame, the way guitar does it.
pub fn draw(frame: &mut Frame, area: Rect, border: Rect, settings: &mut Settings, context: &Context) {
    let width = content_width(area);
    let body = body(settings.tab(), context, area, width);

    settings.selections = body.selections;
    settings.tab_hitboxes = body.hitboxes;
    settings.snap();

    let visible = area.height as usize;
    let total = body.lines.len();
    settings.trap_scroll(total, visible);

    let start = settings.scroll.min(total.saturating_sub(visible));
    let end = (start + visible).min(total);
    let theme = context.theme;

    let items: Vec<ListItem> = body.lines[start..end]
        .iter()
        .enumerate()
        .map(|(offset, line)| {
            let mut line = line.clone();
            // A blank line still has to occupy its row, or the shading behind it
            // collapses and the rhythm of the sections goes with it.
            if line.spans.is_empty() {
                line.spans.push(Span::raw(" "));
            }
            if start + offset == settings.selected {
                let spans: Vec<Span> = line.spans.iter().map(|span| Span::styled(span.content.clone(), span.style.bg(theme.background_or_default(theme.COLOR_GREY_800)))).collect();
                line = Line::from(spans).centered();
            }
            ListItem::from(line)
        })
        .collect();

    frame.render_widget(List::new(items).style(theme.background_style()), area);

    let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("╮"))
        .end_symbol(Some("╯"))
        .track_symbol(Some("│"))
        .thumb_symbol("▌")
        .track_style(Style::default().fg(theme.COLOR_BORDER))
        .thumb_style(Style::default().fg(theme.COLOR_GREY_600));
    frame.render_stateful_widget(scrollbar, border, &mut ScrollbarState::new(scroll::content_length(total, visible)).position(start));
}

/// Everything the view draws, and where the parts that can be landed on ended
/// up. Built in one pass so drawing, scrolling and clicking cannot disagree.
struct Body {
    lines: Vec<Line<'static>>,
    selections: Vec<Selection>,
    hitboxes: Vec<TabHitbox>,
}

impl Body {
    fn blank(&mut self) {
        self.lines.push(Line::default());
    }

    fn push(&mut self, line: Line<'static>) {
        self.lines.push(line);
    }

    /// Marks the line just pushed as one the cursor can land on.
    fn selectable(&mut self, kind: SelectionKind) {
        self.selections.push(Selection { line: self.lines.len().saturating_sub(1), kind });
    }
}

fn content_width(area: Rect) -> usize {
    (area.width as usize).saturating_sub(1).saturating_sub(MARGIN).min(MAX_CONTENT_WIDTH)
}

/// Where a centred row of this width starts on screen, which is what a tab
/// hitbox has to be measured from.
fn centred_start(area: Rect, width: usize) -> u16 {
    area.x + area.width.saturating_sub(width as u16) / 2
}

/// A section heading: the label padded out to the column, in the highlight
/// colour, and always between two blank lines -- that is guitar's divider.
fn section(label: &str, width: usize, theme: &Theme) -> Line<'static> {
    Line::from(Span::styled(fill_width(label, "", width), Style::default().fg(theme.COLOR_HIGHLIGHTED))).centered()
}

/// `label            value`, filling the column exactly.
fn row(left: &str, right: &str, width: usize, style: Style) -> Line<'static> {
    Line::from(Span::styled(fill_width(left, right, width), style)).centered()
}

/// Rows alternate a shaded background, which is what separates them without a
/// rule between every pair.
fn shade(index: usize, theme: &Theme) -> Style {
    let style = Style::default().fg(theme.COLOR_TEXT);
    if index.is_multiple_of(2) { style.bg(theme.background_or_default(theme.COLOR_GREY_900)) } else { style }
}

fn body(tab: Tab, context: &Context, area: Rect, width: usize) -> Body {
    let mut body = Body { lines: Vec::new(), selections: Vec::new(), hitboxes: Vec::new() };

    header(&mut body, tab, context, area, width);
    match tab {
        Tab::General => general(&mut body, context, width),
        Tab::Display => display(&mut body, context, width),
        Tab::Profiles => profiles(&mut body, context, width),
        Tab::Shortcuts => shortcuts(&mut body, context, width),
    }

    body
}

/// Blank, version, blank, wordmark, blank, tab bar, blank -- guitar's header,
/// with the wordmark standing where its heatmap stands.
fn header(body: &mut Body, tab: Tab, context: &Context, area: Rect, width: usize) {
    let theme = context.theme;

    body.blank();
    body.push(row(" version:", &format!("{VERSION} "), width, shade(0, theme)));
    body.selectable(SelectionKind::Info);

    body.blank();
    let rows = logo::rows_for(width);
    for (index, line) in rows.iter().enumerate() {
        let colour = logo::tone(index, rows, theme);
        body.push(Line::from(Span::styled(*line, Style::default().fg(colour))).centered());
    }

    body.blank();
    tab_bar(body, tab, context, area, width);
    body.blank();
}

fn tab_bar(body: &mut Body, current: Tab, context: &Context, area: Rect, width: usize) {
    let theme = context.theme;
    let line = body.lines.len();

    // Full labels when they fit, a dot each when they do not -- guitar narrows
    // the same way rather than letting the bar overflow.
    let full: Vec<String> = Tab::ALL.iter().map(|tab| format!(" {} ", tab.label())).collect();
    let full_width = full.iter().map(|label| label.chars().count()).sum::<usize>() + TAB_GAP.len() * (full.len() - 1);
    let (labels, gap) = if full_width <= width { (full, TAB_GAP) } else { (Tab::ALL.iter().map(|_| COMPACT_TAB.to_owned()).collect(), " ") };

    let bar_width = labels.iter().map(|label| label.chars().count()).sum::<usize>() + gap.chars().count() * (labels.len() - 1);
    let pad = width.saturating_sub(bar_width);
    let left_pad = pad / 2;
    let start = centred_start(area, bar_width + pad);

    let mut spans = vec![Span::raw(" ".repeat(left_pad))];
    let mut offset = left_pad;
    for (index, (tab, label)) in Tab::ALL.iter().zip(&labels).enumerate() {
        if index > 0 {
            spans.push(Span::raw(gap));
            offset += gap.chars().count();
        }

        let label_width = label.chars().count();
        body.hitboxes.push(TabHitbox { tab: *tab, line, start: start + offset as u16, end: start + (offset + label_width) as u16 });

        let style = if *tab == current { Style::default().fg(theme.COLOR_HIGHLIGHTED).bg(theme.background_or_default(theme.COLOR_GREY_900)) } else { Style::default().fg(theme.COLOR_TEXT) };
        spans.push(Span::styled(label.clone(), style));
        offset += label_width;
    }
    spans.push(Span::raw(" ".repeat(pad.saturating_sub(left_pad))));

    body.push(Line::from(spans).centered());
}

fn general(body: &mut Body, context: &Context, width: usize) {
    let theme = context.theme;

    body.blank();
    body.push(section(" paths:", width, theme));
    body.blank();
    let mut paths = vec![(" config:", Config::path()), (" profiles:", profiles_file::path()), (" theme:", palette::theme_path()), (" layout:", layout_config::path())];
    // The one file atrium writes outside its own directory, and only worth
    // naming on a machine that has the opencode it is written for.
    if context.installed.iter().any(|found| found.program == "opencode") {
        paths.push((" opencode theme:", opencode::theme_path()));
    }
    for (index, (label, path)) in paths.iter().enumerate() {
        body.push(row(label, &format!("{} ", path.display()), width, shade(index, theme)));
        body.selectable(SelectionKind::Info);
    }

    body.blank();
    body.push(section(" projects:", width, theme));
    body.blank();
    body.push(row(" root:", &format!("{} ", projects::default_root().display()), width, shade(0, theme)));
    body.selectable(SelectionKind::Info);
    body.push(row(" socket:", &format!("{} ", context.socket.display()), width, shade(1, theme)));
    body.selectable(SelectionKind::Info);
}

fn display(body: &mut Body, context: &Context, width: usize) {
    let theme = context.theme;

    body.blank();
    body.push(section(" themes:", width, theme));
    body.blank();
    for (index, preset) in THEME_PRESETS.iter().enumerate() {
        let marker = if preset.theme.name == theme.name { RADIO_ON } else { RADIO_OFF };
        body.push(row(&format!(" {}", preset.label), &format!("{marker} "), width, shade(index, theme)));
        body.selectable(SelectionKind::Theme(index));
    }
}

fn profiles(body: &mut Body, context: &Context, width: usize) {
    let theme = context.theme;
    let plain = Style::default().fg(theme.COLOR_TEXT);

    body.blank();
    body.push(section(" profiles:", width, theme));
    body.blank();
    body.push(row(" actions:", "select to manage | + add to create ", width, plain));
    body.blank();

    body.push(row(" + add profile", "(enter) ", width, Style::default().fg(theme.COLOR_GRASS).bg(theme.background_or_default(theme.COLOR_GREY_900))));
    body.selectable(SelectionKind::AddProfile);

    if context.profiles.is_empty() {
        body.push(row("  none yet -- what is installed below is what is on offer", "", width, plain));
    }

    // One name column across every row, so the directories line up under each
    // other rather than each row finding its own edge.
    let name_width = context.profiles.profiles.iter().map(|profile| profile.name.chars().count()).max().unwrap_or(0);

    for (index, profile) in context.profiles.profiles.iter().enumerate() {
        let is_default = index == context.default_profile;
        let marker = if is_default { RADIO_ON } else { RADIO_OFF };
        let mut style = Style::default().fg(if is_default { theme.COLOR_GRASS } else { theme.COLOR_TEXT });
        // Offset by one: the add row above is part of the same run of stripes.
        if (index + 1).is_multiple_of(2) {
            style = style.bg(theme.background_or_default(theme.COLOR_GREY_900));
        }

        let left = format!(" {:<name_width$}   {}", profile.name, profile.config_dir);
        body.push(row(&truncate_with_ellipsis(&left, width.saturating_sub(4)), &format!("{marker} "), width, style));
        body.selectable(SelectionKind::Profile(index));
    }

    installed(body, context, width);
}

/// What the `PATH` scan found. It belongs beside the profiles because the two
/// together are what the picker offers: the rows above are what you wrote down,
/// these are what the machine turned out to have.
fn installed(body: &mut Body, context: &Context, width: usize) {
    let theme = context.theme;

    body.blank();
    body.push(section(" installed:", width, theme));
    body.blank();

    for (index, program) in KNOWN_PROGRAMS.iter().enumerate() {
        let found = context.installed.iter().find(|found| found.program == *program);
        // A missing one is said quietly rather than left out: "why is codex not
        // on the splash" is a question the list should answer by itself.
        let style = if found.is_some() { shade(index, theme) } else { shade(index, theme).fg(theme.COLOR_GREY_600) };
        let right = found.map_or_else(|| "not installed ".to_owned(), |found| format!("{} ", found.path.display()));

        body.push(row(&format!(" {program}"), &right, width, style));
        body.selectable(SelectionKind::Info);
    }
}

fn shortcuts(body: &mut Body, context: &Context, width: usize) {
    let theme = context.theme;

    body.blank();
    body.push(section(" keys:", width, theme));
    body.blank();
    for (index, (action, chord)) in context.keymap.actions().iter().enumerate() {
        body.push(row(&format!(" {action}"), &format!("{} ", context.keymap.gesture(*chord)), width, shade(index, theme)));
        body.selectable(SelectionKind::Chord(index));
    }
}

#[cfg(test)]
#[path = "../../tests/app/draw/settings.rs"]
mod tests;
