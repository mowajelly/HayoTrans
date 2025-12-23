//! Application configuration stored in INI format
//!
//! Configuration file is stored next to the executable.

use ini::Ini;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const CONFIG_FILENAME: &str = "hayotrans.ini";

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// UI language (en, ko)
    pub language: String,
    /// Theme (light, dark, system)
    pub theme: String,
    /// Last opened project ID
    pub last_project_id: Option<String>,
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
    /// Max line length for dialogue injection
    pub max_line_length: Option<usize>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            theme: "system".to_string(),
            last_project_id: None,
            window_width: 1280,
            window_height: 800,
            max_line_length: Some(55),
        }
    }
}

impl AppConfig {
    /// Get the configuration file path (next to executable)
    pub fn config_path() -> PathBuf {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf()))
            .unwrap_or_else(|| PathBuf::from("."))
            .join(CONFIG_FILENAME)
    }

    /// Load configuration from INI file
    pub fn load() -> Self {
        let path = Self::config_path();
        tracing::info!("Config file path: {:?}", path);
        
        if !path.exists() {
            tracing::info!("Config file does not exist, creating default");
            let config = Self::default();
            let _ = config.save(); // Try to create default config
            return config;
        }

        tracing::info!("Loading config from: {:?}", path);

        match Ini::load_from_file(&path) {
            Ok(ini) => Self::from_ini(&ini),
            Err(e) => {
                tracing::warn!("Failed to load config: {}, using defaults", e);
                Self::default()
            }
        }
    }

    /// Save configuration to INI file
    pub fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        let ini = self.to_ini();
        
        ini.write_to_file(&path)
            .map_err(|e| format!("Failed to save config: {}", e))
    }

    /// Parse configuration from INI
    fn from_ini(ini: &Ini) -> Self {
        let general = ini.section(Some("General"));
        let window = ini.section(Some("Window"));
        let translation = ini.section(Some("Translation"));

        Self {
            language: general
                .and_then(|s| s.get("language"))
                .unwrap_or("en")
                .to_string(),
            theme: general
                .and_then(|s| s.get("theme"))
                .unwrap_or("system")
                .to_string(),
            last_project_id: general
                .and_then(|s| s.get("last_project_id"))
                .map(|s| s.to_string()),
            window_width: window
                .and_then(|s| s.get("width"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(1280),
            window_height: window
                .and_then(|s| s.get("height"))
                .and_then(|s| s.parse().ok())
                .unwrap_or(800),
            max_line_length: translation
                .and_then(|s| s.get("max_line_length"))
                .and_then(|s| s.parse().ok()),
        }
    }

    /// Convert configuration to INI
    fn to_ini(&self) -> Ini {
        let mut ini = Ini::new();

        ini.with_section(Some("General"))
            .set("language", &self.language)
            .set("theme", &self.theme);

        if let Some(ref id) = self.last_project_id {
            ini.with_section(Some("General"))
                .set("last_project_id", id);
        }

        ini.with_section(Some("Window"))
            .set("width", self.window_width.to_string())
            .set("height", self.window_height.to_string());

        if let Some(max_len) = self.max_line_length {
            ini.with_section(Some("Translation"))
                .set("max_line_length", max_len.to_string());
        }

        ini
    }

    /// Update language setting
    pub fn set_language(&mut self, language: &str) -> Result<(), String> {
        self.language = language.to_string();
        self.save()
    }

    /// Update theme setting
    pub fn set_theme(&mut self, theme: &str) -> Result<(), String> {
        self.theme = theme.to_string();
        self.save()
    }

    /// Update last project ID
    pub fn set_last_project(&mut self, project_id: Option<&str>) -> Result<(), String> {
        self.last_project_id = project_id.map(|s| s.to_string());
        self.save()
    }

    /// Update window size
    pub fn set_window_size(&mut self, width: u32, height: u32) -> Result<(), String> {
        self.window_width = width;
        self.window_height = height;
        self.save()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.language, "en");
        assert_eq!(config.theme, "system");
        assert_eq!(config.window_width, 1280);
    }

    #[test]
    fn test_config_to_ini_and_back() {
        let config = AppConfig {
            language: "ko".to_string(),
            theme: "dark".to_string(),
            last_project_id: Some("project-123".to_string()),
            window_width: 1920,
            window_height: 1080,
            max_line_length: Some(60),
        };

        let ini = config.to_ini();
        let parsed = AppConfig::from_ini(&ini);

        assert_eq!(parsed.language, "ko");
        assert_eq!(parsed.theme, "dark");
        assert_eq!(parsed.last_project_id, Some("project-123".to_string()));
        assert_eq!(parsed.window_width, 1920);
        assert_eq!(parsed.window_height, 1080);
        assert_eq!(parsed.max_line_length, Some(60));
    }
}
