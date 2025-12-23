use std::path::PathBuf;

use crate::types::DetectionResult;
use crate::retriever::{GameDetector, RpgMakerDetector};

/// 게임 디렉토리에서 엔진 감지
#[tauri::command]
pub fn detect_game_engine(path: String) -> DetectionResult {
    tracing::info!("Command: detect_game_engine - path: {}", path);
    
    let path = PathBuf::from(path);
    GameDetector::detect(&path)
}

/// 게임 엔진이 지원되는지 확인
#[tauri::command]
pub fn is_game_supported(path: String) -> bool {
    tracing::info!("Command: is_game_supported - path: {}", path);
    
    let path = PathBuf::from(path);
    GameDetector::is_supported(&path)
}

/// RPG Maker 프로젝트 파일 생성
#[tauri::command]
pub fn create_rpg_maker_project_file(
    rgss_file: String,
    output_dir: String,
) -> std::result::Result<String, String> {
    tracing::info!(
        "Command: create_rpg_maker_project_file - rgss_file: {}, output_dir: {}",
        rgss_file,
        output_dir
    );
    
    let rgss_path = PathBuf::from(rgss_file);
    let output_path = PathBuf::from(output_dir);
    
    match RpgMakerDetector::create_project_file(&rgss_path, &output_path) {
        Ok(created_path) => Ok(created_path.to_string_lossy().to_string()),
        Err(e) => Err(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_game_engine_command() {
        let result = detect_game_engine("/nonexistent".to_string());
        assert!(!result.success);
    }

    #[test]
    fn test_is_game_supported_command() {
        let result = is_game_supported("/nonexistent".to_string());
        assert!(!result);
    }
}
