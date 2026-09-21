pub mod adapters;
pub mod app;
pub mod core {
    pub mod agent;
    pub mod config;
    pub mod git;
    pub mod installed;
    pub mod layout_config;
    pub mod notify;
    pub mod profile;
    pub mod profiles_file;
    pub mod projects;
    pub mod pty;
    pub mod registry;
    pub mod tmux;
    pub mod worktree;
}
pub mod helpers {
    pub mod json;
    pub mod logo;
    pub mod palette;
    pub mod scroll;
    pub mod shell;
    pub mod spinner;
    pub mod text;
    pub mod version;
}
pub mod ipc {
    pub mod hook;
    pub mod server;
    pub mod wire;
}

pub use app::app::App;
pub use core::{agent::AgentSpec, config::Config};
pub use helpers::version::VERSION;

#[cfg(test)]
#[path = "tests/tree.rs"]
mod tree_tests;
