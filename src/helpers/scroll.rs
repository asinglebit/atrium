/// Nudges the offset just far enough to keep `selected` on screen, and never
/// further than the end of the list.
pub fn trap(selected: usize, scroll: usize, total: usize, visible: usize) -> usize {
    if visible == 0 || total <= visible {
        return 0;
    }
    let furthest = total - visible;
    let mut scroll = scroll.min(furthest);
    if selected < scroll {
        scroll = selected;
    } else if selected >= scroll + visible {
        scroll = selected + 1 - visible;
    }
    scroll.min(furthest)
}

/// How many scroll positions there are, which is what a ratatui scrollbar wants
/// as its content length. Zero when everything already fits.
pub fn content_length(total: usize, visible: usize) -> usize {
    if total > visible { total - visible + 1 } else { 0 }
}

#[cfg(test)]
#[path = "../tests/helpers/scroll.rs"]
mod tests;
