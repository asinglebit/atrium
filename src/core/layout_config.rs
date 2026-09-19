use std::{
    fs,
    path::{Path, PathBuf},
};

use facet::Facet;

use crate::app::state::layout::SIDEBAR_WIDTH;

/// What atrium remembers about its own shape. Written by atrium, never by hand
/// -- `config.toml` is the file you edit, and atrium never writes that one.
#[derive(Facet, Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutConfig {
    #[facet(default = SIDEBAR_WIDTH)]
    pub sidebar_width: u16,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self { sidebar_width: SIDEBAR_WIDTH }
    }
}

/// Beside `theme.json`, which is the other file atrium writes for itself.
pub fn path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("atrium");
    path.push("layout.json");
    path
}

pub fn load() -> LayoutConfig {
    load_from(&path())
}

pub fn save(config: &LayoutConfig) {
    save_to(&path(), config);
}

/// A file that is missing, unreadable or malformed is not a problem: the
/// defaults are a perfectly good answer, and a bad one gets overwritten by the
/// next drag.
pub fn load_from(path: &Path) -> LayoutConfig {
    fs::read_to_string(path).ok().and_then(|text| facet_json::from_str::<LayoutConfig>(&text).ok()).unwrap_or_default()
}

/// Nothing here is worth failing over. Losing a dragged width costs one drag;
/// taking atrium down over it costs an agent.
pub fn save_to(path: &Path, config: &LayoutConfig) {
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(text) = facet_json::to_string_pretty(config) {
        let _ = fs::write(path, text);
    }
}

#[cfg(test)]
#[path = "../tests/core/layout_config.rs"]
mod tests;
