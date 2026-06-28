use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct Settings {
    pub last_folder: Option<PathBuf>,
    pub recent_folders: Vec<PathBuf>,
    pub recent_files: Vec<PathBuf>,
    pub ignored_extension_warnings: Vec<String>,
    pub zoom_level: f32,
    pub window_size: Option<[f32; 2]>,
    pub last_position: usize,
    pub auto_restore: bool,
}

impl Settings {
    pub fn load() -> Self {
        let config_dir = Self::config_dir();

        let config_file = config_dir.join("settings.json");

        if config_file.exists() {
            if let Ok(data) = fs::read_to_string(&config_file) {
                if let Ok(settings) = serde_json::from_str(&data) {
                    return settings;
                }
            }
        }

        Self {
            auto_restore: true,
            ..Default::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_dir = Self::config_dir();
        fs::create_dir_all(&config_dir)?;

        let config_file = config_dir.join("settings.json");
        let data = serde_json::to_string_pretty(self)?;
        fs::write(config_file, data)?;

        Ok(())
    }

    fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("photo_exit")
    }

    /// 添加最近打开的文件夹（去重，最多保留 10 个）
    pub fn add_recent_folder(&mut self, path: PathBuf) {
        self.recent_folders.retain(|p| p != &path);
        self.recent_folders.insert(0, path);
        if self.recent_folders.len() > 10 {
            self.recent_folders.truncate(10);
        }
    }

    /// 添加最近打开的文件（去重，最多保留 10 个）
    pub fn add_recent_file(&mut self, path: PathBuf) {
        self.recent_files.retain(|p| p != &path);
        self.recent_files.insert(0, path);
        if self.recent_files.len() > 10 {
            self.recent_files.truncate(10);
        }
    }

    /// 获取有效的最近文件夹列表
    pub fn valid_recent_folders(&self) -> Vec<&PathBuf> {
        self.recent_folders.iter().filter(|p| p.exists()).collect()
    }

    /// 获取有效的最近文件列表
    pub fn valid_recent_files(&self) -> Vec<&PathBuf> {
        self.recent_files.iter().filter(|p| p.exists()).collect()
    }
}
