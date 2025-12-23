use std::path::Path;

use crate::types::DetectionResult;
use super::{rpg_maker::RpgMakerDetector, kirikiri::KiriKiriDetector, v8_engine::V8EngineDetector};

/// 게임 엔진 통합 감지기
pub struct GameDetector;

impl GameDetector {
    /// 디렉토리에서 게임 엔진 자동 감지
    pub fn detect(path: &Path) -> DetectionResult {
        tracing::info!("Starting game engine detection at: {:?}", path);

        if !path.exists() {
            return DetectionResult::failure("Directory does not exist")
                .add_detail(format!("Path: {:?}", path));
        }

        if !path.is_dir() {
            return DetectionResult::failure("Path is not a directory")
                .add_detail(format!("Path: {:?}", path));
        }

        let mut details = Vec::new();

        // 1. RPG Maker 감지 시도
        details.push("Checking for RPG Maker...".to_string());
        match RpgMakerDetector::detect(path) {
            Ok(Some(project)) => {
                details.push(format!("✓ Detected: {}", project.engine.name()));
                return DetectionResult::success(project).with_details(details);
            }
            Ok(None) => {
                details.push("✗ Not an RPG Maker project".to_string());
            }
            Err(e) => {
                details.push(format!("✗ RPG Maker detection error: {}", e));
            }
        }

        // 2. KiriKiri 감지 시도
        details.push("Checking for KiriKiri...".to_string());
        match KiriKiriDetector::detect(path) {
            Ok(Some(project)) => {
                details.push(format!("✓ Detected: {}", project.engine.name()));
                return DetectionResult::success(project).with_details(details);
            }
            Ok(None) => {
                details.push("✗ Not a KiriKiri project".to_string());
            }
            Err(e) => {
                details.push(format!("✗ KiriKiri detection error: {}", e));
            }
        }

        // 3. V8 엔진 감지 시도
        details.push("Checking for V8 Engine...".to_string());
        match V8EngineDetector::detect(path) {
            Ok(Some(project)) => {
                details.push(format!("✓ Detected: {}", project.engine.name()));
                return DetectionResult::success(project).with_details(details);
            }
            Ok(None) => {
                details.push("✗ Not a V8 Engine project".to_string());
            }
            Err(e) => {
                details.push(format!("✗ V8 Engine detection error: {}", e));
            }
        }

        // 감지 실패
        details.push("✗ Unknown or unsupported game engine".to_string());
        DetectionResult::failure("Could not detect game engine")
            .with_details(details)
    }

    /// 여러 디렉토리에서 게임 엔진 감지 (일괄 처리)
    pub fn detect_batch(paths: Vec<&Path>) -> Vec<DetectionResult> {
        paths.into_iter().map(|path| Self::detect(path)).collect()
    }

    /// 게임 엔진이 지원되는지 확인
    pub fn is_supported(path: &Path) -> bool {
        match Self::detect(path) {
            result if result.success => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::env;

    #[test]
    fn test_detect_nonexistent_directory() {
        let path = Path::new("/nonexistent/directory");
        let result = GameDetector::detect(path);
        
        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_is_supported() {
        let temp_dir = env::temp_dir();
        let result = GameDetector::is_supported(&temp_dir);
        
        // temp_dir는 게임 프로젝트가 아니므로 false
        assert!(!result);
    }

    #[test]
    fn test_detect_batch() {
        let paths = vec![
            Path::new("/path1"),
            Path::new("/path2"),
        ];
        
        let results = GameDetector::detect_batch(paths);
        assert_eq!(results.len(), 2);
    }
}
