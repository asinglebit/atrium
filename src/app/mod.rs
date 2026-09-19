#[allow(clippy::module_inception)]
pub mod app;

pub mod draw {
    pub mod modals {
        pub mod new_agent;
    }
    pub mod sidebar;
    pub mod stage;
}

pub mod input {
    pub mod keymap;
    pub mod keys;
}

pub mod state {
    pub mod layout;
    pub mod picker;
}
