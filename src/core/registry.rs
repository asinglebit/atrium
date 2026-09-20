use std::{io, path::PathBuf};

use crate::{
    core::agent::{Agent, Status},
    ipc::wire::Report,
};

/// How many agents are held, and how many of them are in each state worth
/// counting. What the bar segment outside atrium shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub held: usize,
    pub working: usize,
    pub needs_input: usize,
    pub error: usize,
}

/// The agents this atrium is holding, and which one the stage is showing.
#[derive(Default)]
pub struct Registry {
    agents: Vec<Agent>,
    focus: usize,
}

impl Registry {
    pub fn new() -> Self {
        Self::default()
    }

    /// A newly held agent takes the stage, which is what you want every time.
    pub fn push(&mut self, agent: Agent) {
        self.agents.push(agent);
        self.focus = self.agents.len() - 1;
    }

    pub fn agents(&self) -> &[Agent] {
        &self.agents
    }

    pub fn len(&self) -> usize {
        self.agents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    pub fn focus(&self) -> usize {
        self.focus
    }

    pub fn focused(&self) -> Option<&Agent> {
        self.agents.get(self.focus)
    }

    pub fn focused_mut(&mut self) -> Option<&mut Agent> {
        self.agents.get_mut(self.focus)
    }

    pub fn focus_next(&mut self) {
        if !self.agents.is_empty() {
            self.focus = (self.focus + 1) % self.agents.len();
        }
    }

    pub fn focus_prev(&mut self) {
        if !self.agents.is_empty() {
            self.focus = (self.focus + self.agents.len() - 1) % self.agents.len();
        }
    }

    /// Out-of-range jumps are ignored rather than clamped, so a stray `F12 7`
    /// does not move the stage somewhere you did not ask for.
    pub fn focus_at(&mut self, index: usize) {
        if index < self.agents.len() {
            self.focus = index;
        }
    }

    pub fn dismiss_focused(&mut self) -> Option<Agent> {
        self.dismiss_at(self.focus)
    }

    /// Ends one agent by row. Dropping the `Agent` is what kills it.
    pub fn dismiss_at(&mut self, index: usize) -> Option<Agent> {
        if index >= self.agents.len() {
            return None;
        }
        let agent = self.agents.remove(index);
        // A row removed above the focused one shifts it down, so the focus
        // follows the agent it was on rather than sliding to its neighbour.
        if index < self.focus {
            self.focus -= 1;
        }
        self.focus = self.focus.min(self.agents.len().saturating_sub(1));
        Some(agent)
    }

    /// Every agent is resized, not just the focused one, so switching to a
    /// background agent never shows a stale layout.
    pub fn resize_all(&mut self, rows: u16, cols: u16) -> io::Result<()> {
        for agent in &mut self.agents {
            agent.resize(rows, cols)?;
        }
        Ok(())
    }

    /// Reports name an agent by id, so they land correctly even after the rows
    /// have been reordered or dismissed.
    pub fn apply(&mut self, report: &Report) {
        if let Some(agent) = self.agents.iter_mut().find(|agent| agent.id == report.agent_id) {
            agent.apply_event(&report.event);
        }
    }

    pub fn refresh_git(&mut self) {
        for agent in &mut self.agents {
            agent.refresh_git();
        }
    }

    pub fn refresh(&mut self) {
        for agent in &mut self.agents {
            agent.refresh_status();
        }
    }

    /// The worst of what the held agents are doing -- what this atrium needs,
    /// in one word, for whatever is showing it from outside.
    ///
    /// `Exited` is left out: a row that has finished is not something to be
    /// pulled back to, and an atrium holding nothing but dead agents needs
    /// nothing.
    pub fn aggregate(&self) -> Option<Status> {
        self.agents.iter().map(|agent| agent.status).filter(|status| *status != Status::Exited).max_by_key(|status| match status {
            Status::Error => 4,
            Status::NeedsInput => 3,
            Status::Working => 2,
            Status::Idle => 1,
            Status::Exited => 0,
        })
    }

    /// How many are held and what they are doing. `held` counts the dead ones
    /// too, because a row is still a row.
    pub fn counts(&self) -> Counts {
        let mut counts = Counts { held: self.agents.len(), ..Counts::default() };
        for agent in &self.agents {
            match agent.status {
                Status::Working => counts.working += 1,
                Status::NeedsInput => counts.needs_input += 1,
                Status::Error => counts.error += 1,
                Status::Idle | Status::Exited => {},
            }
        }
        counts
    }

    /// The distinct directories the held agents stand in, which is where to
    /// look for worktrees that appeared without anyone saying so.
    pub fn repos(&self) -> Vec<PathBuf> {
        let mut repos: Vec<PathBuf> = self.agents.iter().map(|agent| agent.cwd().to_path_buf()).collect();
        repos.sort();
        repos.dedup();
        repos
    }
}

#[cfg(test)]
#[path = "../tests/core/registry.rs"]
mod tests;
