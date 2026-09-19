use std::io;

use crate::{core::agent::Agent, ipc::wire::Report};

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
        if self.agents.is_empty() {
            return None;
        }
        let agent = self.agents.remove(self.focus);
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
}

#[cfg(test)]
#[path = "../tests/core/registry.rs"]
mod tests;
