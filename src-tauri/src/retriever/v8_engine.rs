use std::path::Path;
use std::fs;

use crate::types::{
    GameEngine, V8Engine, ProjectMetadata, GameProject, Result,
};

/// V8 엔진 프로젝트 감지기
pub struct V8EngineDetector;

impl V8EngineDetector {
    /// 디렉토리에서 V8 엔진 프로젝트 감지
    pub fn detect(path: &Path) -> Result<Option<GameProject>> {
        tracing::info!("Detecting V8 Engine project at: {:?}", path);

        // package.json 확인
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }

        let content = match fs::read_to_string(&package_json_path) {
            Ok(content) => content,
            Err(_) => return Ok(None),
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(json) => json,
            Err(_) => return Ok(None),
        };

        // V8 엔진 타입 감지
        let engine_type = Self::detect_engine_type(&json, path)?;
        if engine_type.is_none() {
            return Ok(None);
        }

        let engine_type = engine_type.unwrap();
        let metadata = Self::extract_metadata(&json);

        Ok(Some(GameProject::new(
            path.to_path_buf(),
            GameEngine::V8Engine(engine_type),
            Self::get_version(&json),
            metadata,
        )))
    }

    /// V8 엔진 타입 감지
    fn detect_engine_type(json: &serde_json::Value, path: &Path) -> Result<Option<V8Engine>> {
        // NW.js 감지
        if json.get("nw").is_some() || json.get("node-webkit").is_some() {
            tracing::info!("Detected NW.js project");
            return Ok(Some(V8Engine::NwJs));
        }

        // Electron 감지
        if json.get("electron").is_some() {
            tracing::info!("Detected Electron project");
            return Ok(Some(V8Engine::Electron));
        }

        // dependencies에서 확인
        if let Some(deps) = json.get("dependencies").and_then(|v| v.as_object()) {
            if deps.contains_key("nw") {
                tracing::info!("Detected NW.js project (from dependencies)");
                return Ok(Some(V8Engine::NwJs));
            }
            if deps.contains_key("electron") {
                tracing::info!("Detected Electron project (from dependencies)");
                return Ok(Some(V8Engine::Electron));
            }
        }

        // devDependencies에서 확인
        if let Some(dev_deps) = json.get("devDependencies").and_then(|v| v.as_object()) {
            if dev_deps.contains_key("nw") {
                tracing::info!("Detected NW.js project (from devDependencies)");
                return Ok(Some(V8Engine::NwJs));
            }
            if dev_deps.contains_key("electron") {
                tracing::info!("Detected Electron project (from devDependencies)");
                return Ok(Some(V8Engine::Electron));
            }
        }

        // 실행 파일로 확인
        if path.join("nw.exe").exists() || path.join("nw").exists() {
            tracing::info!("Detected NW.js project (from executable)");
            return Ok(Some(V8Engine::NwJs));
        }

        if path.join("electron.exe").exists() || path.join("electron").exists() {
            tracing::info!("Detected Electron project (from executable)");
            return Ok(Some(V8Engine::Electron));
        }

        // resources/app.asar 확인 (Electron)
        if path.join("resources/app.asar").exists() {
            tracing::info!("Detected Electron project (from app.asar)");
            return Ok(Some(V8Engine::Electron));
        }

        // main 필드에서 확인
        if let Some(main) = json.get("main").and_then(|v| v.as_str()) {
            if main.contains("electron") {
                tracing::info!("Detected Electron project (from main field)");
                return Ok(Some(V8Engine::Electron));
            }
        }

        Ok(None)
    }

    /// 메타데이터 추출
    fn extract_metadata(json: &serde_json::Value) -> ProjectMetadata {
        let title = json
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let author = json
            .get("author")
            .and_then(|v| v.as_str())
            .or_else(|| {
                json.get("author")
                    .and_then(|v| v.get("name"))
                    .and_then(|v| v.as_str())
            })
            .map(|s| s.to_string());

        let description = json
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        ProjectMetadata::new()
            .with_title(title.unwrap_or_else(|| "Unknown".to_string()))
            .with_author(author.unwrap_or_else(|| "Unknown".to_string()))
            .with_description(description.unwrap_or_else(|| "".to_string()))
    }

    /// 버전 정보 추출
    fn get_version(json: &serde_json::Value) -> String {
        json.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("1.0.0")
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_nwjs_from_json() {
        let json_str = r#"{
            "name": "test-game",
            "version": "1.0.0",
            "nw": {
                "version": "0.50.0"
            }
        }"#;
        
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let engine = V8EngineDetector::detect_engine_type(&json, Path::new(".")).unwrap();
        
        assert_eq!(engine, Some(V8Engine::NwJs));
    }

    #[test]
    fn test_detect_electron_from_json() {
        let json_str = r#"{
            "name": "test-game",
            "version": "1.0.0",
            "devDependencies": {
                "electron": "^20.0.0"
            }
        }"#;
        
        let json: serde_json::Value = serde_json::from_str(json_str).unwrap();
        let engine = V8EngineDetector::detect_engine_type(&json, Path::new(".")).unwrap();
        
        assert_eq!(engine, Some(V8Engine::Electron));
    }
}
