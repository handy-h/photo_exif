use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub last_folder: Option<PathBuf>,
    pub ignored_extension_warnings: Vec<String>,
    pub zoom_level: f32,
    pub window_size: Option<[f32; 2]>,
}

impl Settings {
    pub fn load() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("photo_exit");

        let config_file = config_dir.join("settings.json");

        if config_file.exists() {
            if let Ok(data) = fs::read_to_string(&config_file) {
                if let Ok(settings) = serde_json::from_str(&data) {
                    return settings;
                }
            }
        }

        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("photo_exit");

        fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("settings.json");
        let data = serde_json::to_string_pretty(self)?;
        fs::write(config_file, data)?;

        Ok(())
    }
}
