pub mod adapters;
pub mod app;
pub mod core {
    pub mod agent;
    pub mod git;
    pub mod projects;
    pub mod pty;
    pub mod registry;
}
pub mod helpers {
    pub mod json;
    pub mod spinner;
}
pub mod ipc {
    pub mod hook;
    pub mod server;
    pub mod wire;
}

pub use app::app::App;
pub use core::agent::AgentSpec;
