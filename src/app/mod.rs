#[allow(clippy::module_inception)]
pub mod app;

pub mod draw {
    pub mod sidebar;
    pub mod stage;
}

pub mod input {
    pub mod keys;
}

pub mod state {
    pub mod layout;
}
