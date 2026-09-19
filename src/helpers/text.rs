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
