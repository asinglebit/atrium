pub mod app;
pub mod core {
    pub mod agent;
    pub mod pty;
    pub mod registry;
}

pub use app::app::App;
pub use core::agent::AgentSpec;
