#[allow(clippy::module_inception)]
pub mod app;

pub mod draw {
    pub mod modals {
        pub mod goto;
        pub mod new_agent;
        pub mod profile;
    }
    pub mod menu;
    pub mod pane;
    pub mod settings;
    pub mod sidebar;
    pub mod splash;
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
    pub mod mode;
    pub mod picker;
    pub mod profile_editor;
    pub mod settings;
    pub mod splash;
}
