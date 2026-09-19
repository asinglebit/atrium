pub mod adapters;
pub mod app;
pub mod core {
    pub mod agent;
    pub mod config;
    pub mod git;
    pub mod projects;
    pub mod pty;
    pub mod registry;
}
pub mod helpers {
    pub mod json;
    pub mod palette;
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
