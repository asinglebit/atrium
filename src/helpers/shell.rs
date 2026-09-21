/// Wraps a string so a shell reads it as one word, whatever is in it. copilot's
/// hooks take a shell string rather than an argument list, so a path with a
/// space or a quote in it has to be made safe before it goes in one.
///
/// Single quotes, because inside them a shell expands nothing at all. The only
/// thing they cannot hold is another single quote, which is closed, escaped and
/// reopened -- the usual `'\''`.
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', r"'\''"))
}

#[cfg(test)]
#[path = "../tests/helpers/shell.rs"]
mod tests;
