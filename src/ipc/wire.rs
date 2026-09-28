use std::time::{SystemTime, UNIX_EPOCH};

/// What a hook tells atrium: which agent, what just happened, and when. One
/// line, tab separated, because the event name arrives as an argument and there
/// is nothing else to carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub agent_id: u64,
    pub event: String,
    /// When the sender started, in ms since the epoch. Hooks are separate
    /// processes and connect in whatever order they get round to it, so this is
    /// what puts them back in the order they happened. None from a sender that
    /// did not say.
    pub at: Option<u64>,
}

impl Report {
    pub fn encode(&self) -> String {
        match self.at {
            Some(at) => format!("{}\t{}\t{at}\n", self.agent_id, self.event),
            None => format!("{}\t{}\n", self.agent_id, self.event),
        }
    }

    pub fn parse(line: &str) -> Option<Self> {
        let mut fields = line.trim_end_matches(['\r', '\n']).split('\t');
        let agent_id = fields.next()?.trim().parse().ok()?;
        let event = fields.next()?.trim();
        if event.is_empty() {
            return None;
        }
        // A stamp that is there but unreadable is a malformed line, not a
        // missing stamp.
        let at = match fields.next() {
            Some(at) => Some(at.trim().parse().ok()?),
            None => None,
        };
        if fields.next().is_some() {
            return None;
        }
        Some(Self { agent_id, event: event.to_owned(), at })
    }
}

/// Now, the way a report is stamped. The wall clock rather than this process'
/// uptime, because the stamp is compared across processes.
pub fn now_ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |since| u64::try_from(since.as_millis()).unwrap_or(u64::MAX))
}

#[cfg(test)]
#[path = "../tests/ipc/wire.rs"]
mod tests;
