/// Cuts a string to fit, marking it with an ellipsis when something was lost.
pub fn truncate(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    if value.chars().count() <= width {
        return value.to_owned();
    }
    if width == 1 {
        return "…".to_owned();
    }
    value.chars().take(width - 1).chain(std::iter::once('…')).collect()
}

/// Cuts from the front instead of the back, which is what a path wants: the
/// last components are the ones that identify it.
pub fn truncate_start(value: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let len = value.chars().count();
    if len <= width {
        return value.to_owned();
    }
    if width == 1 {
        return "…".to_owned();
    }
    std::iter::once('…').chain(value.chars().skip(len - (width - 1))).collect()
}

/// Cuts to fit with a three-dot ellipsis, which is what guitar's settings rows
/// use. A field too narrow for both content and an ellipsis becomes dots alone.
pub fn truncate_with_ellipsis(value: &str, width: usize) -> String {
    if value.chars().count() <= width {
        return value.to_owned();
    }
    if width <= 3 {
        return ".".repeat(width);
    }
    value.chars().take(width - 3).chain("...".chars()).collect()
}

/// `left`, spaces, `right`, filling exactly `width`. This is what makes every
/// settings row line up: the label sits left, the value sits right, and the gap
/// between them takes whatever is left over.
///
/// There is always at least one space, so a row too full to fill still reads as
/// two fields rather than one run-on word -- it elides instead.
pub fn fill_width(left: &str, right: &str, width: usize) -> String {
    let spaces = width.saturating_sub(left.chars().count() + right.chars().count()).max(1);
    truncate_with_ellipsis(&format!("{left}{}{right}", " ".repeat(spaces)), width)
}

#[cfg(test)]
#[path = "../tests/helpers/text.rs"]
mod tests;
