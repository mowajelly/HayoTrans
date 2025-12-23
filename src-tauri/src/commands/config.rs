//! Tauri commands for application configuration

use crate::storage::AppConfig;
use serde::{Deserialize, Serialize};

/// Config response for frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    pub language: String,
    pub theme: String,
    pub last_project_id: Option<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub max_line_length: Option<usize>,
}

impl From<AppConfig> for ConfigResponse {
    fn from(config: AppConfig) -> Self {
        Self {
            language: config.language,
            theme: config.theme,
            last_project_id: config.last_project_id,
            window_width: config.window_width,
            window_height: config.window_height,
            max_line_length: config.max_line_length,
        }
    }
}

/// Get the current configuration
#[tauri::command]
pub async fn get_config() -> Result<ConfigResponse, String> {
    tracing::info!("Loading config from backend");
    let config = AppConfig::load();
    Ok(ConfigResponse::from(config))
}

/// Set the UI language
#[tauri::command]
pub async fn set_language(language: String) -> Result<(), String> {
    tracing::info!("Setting language to: {}", language);
    let mut config = AppConfig::load();
    config.set_language(&language)
}

/// Set the UI theme
#[tauri::command]
pub async fn set_theme(theme: String) -> Result<(), String> {
    tracing::info!("Setting theme to: {}", theme);
    let mut config = AppConfig::load();
    config.set_theme(&theme)
}

/// Set the last opened project ID
#[tauri::command]
pub async fn set_last_project(project_id: Option<String>) -> Result<(), String> {
    tracing::info!("Setting last project to: {:?}", project_id);
    let mut config = AppConfig::load();
    config.set_last_project(project_id.as_deref())
}

/// Set the window size
#[tauri::command]
pub async fn set_window_size(width: u32, height: u32) -> Result<(), String> {
    tracing::info!("Setting window size to: {}x{}", width, height);
    let mut config = AppConfig::load();
    config.set_window_size(width, height)
}

/// Get the application data directory (where config and database are stored)
#[tauri::command]
pub async fn get_app_data_path() -> Result<String, String> {
    let path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let dir = path.parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;
    
    tracing::info!("App data path: {:?}", dir);
    
    Ok(dir.to_string_lossy().to_string())
}

/// Open the application data directory in the system file explorer
#[tauri::command]
pub async fn open_app_data_folder() -> Result<(), String> {
    let path = std::env::current_exe()
        .map_err(|e| format!("Failed to get executable path: {}", e))?;
    
    let dir = path.parent()
        .ok_or_else(|| "Failed to get parent directory".to_string())?;
    
    tracing::info!("Opening app data folder: {:?}", dir);
    
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map_err(|e| format!("Failed to open folder: {}", e))?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_response_from() {
        let config = AppConfig::default();
        let response = ConfigResponse::from(config);
        assert_eq!(response.language, "en");
        assert_eq!(response.theme, "system");
    }
}
