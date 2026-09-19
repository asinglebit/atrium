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
