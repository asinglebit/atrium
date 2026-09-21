use crossterm::event::KeyEvent;

use crate::app::input::keymap::Keymap;

/// Who the keyboard belongs to. The agent has it by default -- every keystroke
/// reaches it, which is the whole point of holding one -- and atrium takes it
/// only while you have asked for it.
///
/// It is a mode rather than a prefix because the keys worth pressing come in
/// runs: walking the sidebar and dropping a couple of agents is one visit, not
/// five. Guitar splits its own input the same way.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mode {
    /// The focused agent has the keyboard.
    #[default]
    Agent,
    /// atrium has it, and the action keys mean what they say without a prefix.
    Atrium,
}

/// What a key did to the mode, so the caller knows whether the key is still
/// its own to act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    /// atrium now has the keyboard. The key was the way in and nothing else.
    Entered,
    /// The agent has it back. The key was the way out and nothing else.
    Left,
    /// atrium has the keyboard and this key is an action to run.
    Act,
    /// The mode is not on, so the key belongs to whatever had it.
    Pass,
}

impl Mode {
    pub fn is_on(self) -> bool {
        self == Self::Atrium
    }

    /// What this key means. `sticky` is false on the splash, where atrium
    /// already has the keyboard: there is nothing to jump out of, so the chord
    /// reaches the actions once and the mode ends with it.
    pub fn step(&mut self, key: &KeyEvent, keymap: &Keymap, sticky: bool) -> Step {
        if self.is_on() {
            // The chord that opens it closes it, which is the cheapest way out
            // for anyone whose hand is still on it.
            if keymap.action.matches(key) || keymap.leave.matches(key) {
                *self = Self::Agent;
                return Step::Left;
            }
            if !sticky {
                *self = Self::Agent;
            }
            return Step::Act;
        }
        if keymap.action.matches(key) {
            *self = Self::Atrium;
            return Step::Entered;
        }
        Step::Pass
    }

    /// Give the keyboard back. What a surface calls when it takes the keys for
    /// itself -- settings, a modal, the menu, the editor -- and what dropping
    /// the last agent calls, since the splash takes bare keys of its own.
    /// Leaving the mode on underneath one of those would put you back in it on
    /// the way out, which is not where you were going.
    pub fn release(&mut self) {
        *self = Self::Agent;
    }
}

#[cfg(test)]
#[path = "../../tests/app/state/mode.rs"]
mod tests;
