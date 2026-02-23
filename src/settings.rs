use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub current_image_path: Option<String>,
    pub speed: i64,
    pub image_scale: f32,
    pub window_width: f32,
    pub window_height: f32,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            current_image_path: None,
            speed: 0,
            image_scale: 1.0,
            window_width: 400.0,
            window_height: 520.0,
        }
    }
}

impl AppSettings {
    pub fn load() -> Self {
        let path = Self::get_settings_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
                Err(_) => Self::default(),
            }
        } else {
            Self::default()
        }
    }

    pub fn save(&self) {
        let path = Self::get_settings_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let content = serde_json::to_string_pretty(self).unwrap_or_default();
        let _ = fs::write(&path, content);
    }

    fn get_settings_path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        if cfg!(target_os = "macos") {
            home.join("Library/Application Support/AnimaMac/settings.json")
        } else if cfg!(windows) {
            home.join("AppData/Roaming/AnimaMac/settings.json")
        } else {
            home.join(".config/animatux/settings.json")
        }
    }
}
