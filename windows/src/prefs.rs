use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::theme::ThemeMode;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub language: Option<String>,
    pub theme: String,
    pub auto_lock_minutes: u32,
    pub block_capture: bool,
    pub quick_unlock: bool,
    pub hide_codes: bool,
    pub advanced_visible: bool,
    pub animations: bool,
    pub brand_icons: bool,
}

impl Default for Preferences {
    fn default() -> Self {
        Self {
            language: None,
            theme: "system".into(),
            auto_lock_minutes: 5,
            block_capture: true,
            quick_unlock: false,
            hide_codes: false,
            advanced_visible: false,
            animations: true,
            brand_icons: false,
        }
    }
}

impl Preferences {
    pub fn theme_mode(&self) -> ThemeMode {
        match self.theme.as_str() {
            "light" => ThemeMode::Light,
            "dark" => ThemeMode::Dark,
            _ => ThemeMode::System,
        }
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme = match mode {
            ThemeMode::System => "system",
            ThemeMode::Light => "light",
            ThemeMode::Dark => "dark",
        }
        .to_string();
    }

    pub fn load(path: &PathBuf) -> Self {
        std::fs::read_to_string(path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(raw) = serde_json::to_string_pretty(self) {
            let _ = std::fs::write(path, raw);
        }
    }
}

pub struct AppDirs {
    pub vault_path: PathBuf,
    pub prefs_path: PathBuf,
    pub languages_dir: PathBuf,
}

pub fn app_dirs() -> AppDirs {
    let base = directories::ProjectDirs::from("com", "liwidale", "LiAuth")
        .map(|dirs| dirs.data_dir().to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));
    AppDirs {
        vault_path: base.join("vault.liauth"),
        prefs_path: base.join("preferences.json"),
        languages_dir: base.join("languages"),
    }
}
