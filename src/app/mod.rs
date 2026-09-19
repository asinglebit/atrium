#[allow(clippy::module_inception)]
pub mod app;

pub mod draw {
    pub mod modals {
        pub mod goto;
        pub mod new_agent;
    }
    pub mod menu;
    pub mod pane;
    pub mod settings;
    pub mod sidebar;
    pub mod stage;
    pub mod statusbar;
    pub mod title;
}

pub mod input {
    pub mod keymap;
    pub mod keys;
}

pub mod state {
    pub mod goto;
    pub mod layout;
    pub mod menu;
    pub mod picker;
    pub mod settings;
}
