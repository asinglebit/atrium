/// What a hook tells atrium: which agent, and what just happened. One line,
/// tab separated, because the event name arrives as an argument and there is
/// nothing else to carry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub agent_id: u64,
    pub event: String,
}

impl Report {
    pub fn encode(&self) -> String {
        format!("{}\t{}\n", self.agent_id, self.event)
    }

    pub fn parse(line: &str) -> Option<Self> {
        let (id, event) = line.trim_end_matches(['\r', '\n']).split_once('\t')?;
        let event = event.trim();
        if event.is_empty() {
            return None;
        }
        Some(Self { agent_id: id.trim().parse().ok()?, event: event.to_owned() })
    }
}

#[cfg(test)]
#[path = "../tests/ipc/wire.rs"]
mod tests;
