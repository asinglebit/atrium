/// What a profile launches when it does not say otherwise.
pub const DEFAULT_PROGRAM: &str = "claude";

/// The CLIs atrium offers when `config.toml` names no profiles. Each becomes a
/// profile of its own, so the picker has one code path either way.
pub const DEFAULT_PROGRAMS: [&str; 3] = ["claude", "opencode", "codex"];

/// Which Claude subscription is in use is this environment variable and nothing
/// else, which is why `config_dir` is worth its own setting.
pub const CONFIG_DIR_ENV: &str = "CLAUDE_CONFIG_DIR";

/// A named launch recipe: which CLI, plus what to add to its command line and
/// its environment.
///
/// A Claude subscription is exactly one of these — a `CLAUDE_CONFIG_DIR` and
/// whatever flags belong with it. Holding one used to mean a shell alias, which
/// atrium could not reach.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Profile {
    pub name: String,
    pub program: String,
    pub args: Vec<String>,
    pub env: Vec<(String, String)>,
}

impl Profile {
    /// A CLI with nothing added, which is what you get with no config at all.
    pub fn bare(program: &str) -> Self {
        Self { name: program.to_owned(), program: program.to_owned(), args: Vec::new(), env: Vec::new() }
    }

    /// What the picker shows. A profile that only names a CLI says it once.
    pub fn label(&self) -> String {
        if self.name == self.program { self.name.clone() } else { format!("{} · {}", self.program, self.name) }
    }

    /// The name to hang on a sidebar row, which is nothing for a bare CLI --
    /// the row would only be repeating what every other row already says.
    pub fn tag(&self) -> Option<String> {
        (self.name != self.program).then(|| self.name.clone())
    }

    pub fn config_dir(&self) -> Option<&str> {
        self.env.iter().find(|(key, _)| key == CONFIG_DIR_ENV).map(|(_, value)| value.as_str())
    }
}

/// Everything atrium offers when nothing is configured.
pub fn defaults() -> Vec<Profile> {
    DEFAULT_PROGRAMS.iter().map(|program| Profile::bare(program)).collect()
}

/// `~` at the front, and `$VAR` or `${VAR}` anywhere, so a path copied out of a
/// shell alias can be pasted in as it was written rather than hardcoded.
///
/// An unset variable expands to nothing, the way a shell would.
pub fn expand(value: &str) -> String {
    expand_with(value, |name| std::env::var(name).ok())
}

/// The rules, with the lookup handed in. Split out so they can be tested
/// without touching the one environment the whole test binary shares -- setting
/// a variable is process-wide, and the suite runs in parallel.
pub fn expand_with(value: &str, lookup: impl Fn(&str) -> Option<String>) -> String {
    let value = match value.strip_prefix('~') {
        Some(rest) if rest.is_empty() || rest.starts_with('/') => format!("{}{rest}", lookup("HOME").unwrap_or_default()),
        _ => value.to_owned(),
    };
    expand_vars(&value, &lookup)
}

fn expand_vars(value: &str, lookup: &impl Fn(&str) -> Option<String>) -> String {
    let chars: Vec<char> = value.chars().collect();
    let mut out = String::with_capacity(value.len());
    let mut at = 0;

    while at < chars.len() {
        if chars[at] != '$' {
            out.push(chars[at]);
            at += 1;
            continue;
        }

        let braced = chars.get(at + 1) == Some(&'{');
        let start = at + 1 + usize::from(braced);
        let mut end = start;
        // A name starts with a letter or an underscore, the way a shell's does,
        // so `$5` is five dollars rather than a variable called 5.
        if chars.get(end).is_some_and(|c| c.is_ascii_alphabetic() || *c == '_') {
            end += 1;
            while chars.get(end).is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_') {
                end += 1;
            }
        }

        // Nothing that could be a name, or a `${` with nothing closing it: the
        // `$` stays where it was written and the scan carries on past it.
        if end == start || (braced && chars.get(end) != Some(&'}')) {
            out.push('$');
            at += 1;
            continue;
        }

        let name: String = chars[start..end].iter().collect();
        out.push_str(&lookup(&name).unwrap_or_default());
        at = end + usize::from(braced);
    }

    out
}

#[cfg(test)]
#[path = "../tests/core/profile.rs"]
mod tests;
