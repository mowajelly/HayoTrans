use std::path::{Path, PathBuf};
use std::fs;

use crate::types::{
    GameEngine, RpgMakerVersion, ProjectMetadata, GameProject, HayoTransError, Result,
};

/// RPG Maker 프로젝트 감지기
pub struct RpgMakerDetector;

impl RpgMakerDetector {
    /// 디렉토리에서 RPG Maker 프로젝트 감지
    pub fn detect(path: &Path) -> Result<Option<GameProject>> {
        tracing::info!("Detecting RPG Maker project at: {:?}", path);

        // 1. 프로젝트 파일로 감지 (XP/VX/VXAce)
        if let Some(project) = Self::detect_by_project_file(path)? {
            return Ok(Some(project));
        }

        // 2. 데이터 아카이브로 감지 (XP/VX/VXAce)
        if let Some(project) = Self::detect_by_archive(path)? {
            return Ok(Some(project));
        }

        // 3. package.json으로 감지 (MV/MZ)
        if let Some(project) = Self::detect_by_package_json(path)? {
            return Ok(Some(project));
        }

        // 4. www/data 디렉토리로 감지 (MV/MZ)
        if let Some(project) = Self::detect_by_www_directory(path)? {
            return Ok(Some(project));
        }

        Ok(None)
    }

    /// 프로젝트 파일로 감지 (.rxproj, .rvproj, .rvproj2)
    fn detect_by_project_file(path: &Path) -> Result<Option<GameProject>> {
        let project_files = vec![
            ("Game.rxproj", RpgMakerVersion::XP),
            ("Game.rvproj", RpgMakerVersion::VX),
            ("Game.rvproj2", RpgMakerVersion::VXAce),
        ];

        for (filename, version) in project_files {
            let project_path = path.join(filename);
            if project_path.exists() {
                tracing::info!("Found project file: {:?}", project_path);
                
                // 프로젝트 파일 내용 읽기
                let content = fs::read_to_string(&project_path)
                    .map_err(|e| HayoTransError::FileReadError(e.to_string()))?;
                
                let metadata = Self::extract_metadata_from_project_file(&content, version);
                
                return Ok(Some(GameProject::new(
                    path.to_path_buf(),
                    GameEngine::RpgMaker(version),
                    version.to_string(),
                    metadata,
                )));
            }
        }

        Ok(None)
    }

    /// 데이터 아카이브로 감지 (.rgssad, .rgss2a, .rgss3a)
    fn detect_by_archive(path: &Path) -> Result<Option<GameProject>> {
        let entries = fs::read_dir(path)
            .map_err(|e| HayoTransError::DirectoryNotFound(e.to_string()))?;

        for entry in entries.flatten() {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                let ext_with_dot = format!(".{}", ext);
                if let Some(version) = RpgMakerVersion::from_extension(&ext_with_dot) {
                    tracing::info!("Found archive file: {:?}", file_path);
                    
                    let metadata = ProjectMetadata::new()
                        .with_title(
                            file_path
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .unwrap_or("Unknown")
                                .to_string()
                        );
                    
                    return Ok(Some(GameProject::new(
                        path.to_path_buf(),
                        GameEngine::RpgMaker(version),
                        version.to_string(),
                        metadata,
                    )));
                }
            }
        }

        Ok(None)
    }

    /// package.json으로 감지 (MV/MZ)
    fn detect_by_package_json(path: &Path) -> Result<Option<GameProject>> {
        let package_json_path = path.join("package.json");
        if !package_json_path.exists() {
            return Ok(None);
        }

        tracing::info!("Found package.json: {:?}", package_json_path);

        let content = fs::read_to_string(&package_json_path)
            .map_err(|e| HayoTransError::FileReadError(e.to_string()))?;

        let json: serde_json::Value = serde_json::from_str(&content)?;

        // MZ 감지: "rmmz_core" 또는 "rmmz_managers" 등의 파일 확인
        if let Some(main) = json.get("main").and_then(|v| v.as_str()) {
            if main.contains("rmmz") || main.contains("index.html") {
                // www/js/rmmz_core.js 파일 확인
                if path.join("www/js/rmmz_core.js").exists() {
                    let metadata = Self::extract_metadata_from_package_json(&json);
                    return Ok(Some(GameProject::new(
                        path.to_path_buf(),
                        GameEngine::RpgMaker(RpgMakerVersion::MZ),
                        "MZ".to_string(),
                        metadata,
                    )));
                }
            }
        }

        // MV 감지: "rpg_core" 확인
        if path.join("www/js/rpg_core.js").exists() {
            let metadata = Self::extract_metadata_from_package_json(&json);
            return Ok(Some(GameProject::new(
                path.to_path_buf(),
                GameEngine::RpgMaker(RpgMakerVersion::MV),
                "MV".to_string(),
                metadata,
            )));
        }

        Ok(None)
    }

    /// www/data 디렉토리로 감지 (MV/MZ)
    fn detect_by_www_directory(path: &Path) -> Result<Option<GameProject>> {
        let www_data_path = path.join("www/data");
        if !www_data_path.exists() {
            return Ok(None);
        }

        tracing::info!("Found www/data directory: {:?}", www_data_path);

        // System.json 파일 확인
        let system_json_path = www_data_path.join("System.json");
        if !system_json_path.exists() {
            return Ok(None);
        }

        // MZ vs MV 구분
        let version = if path.join("www/js/rmmz_core.js").exists() {
            RpgMakerVersion::MZ
        } else if path.join("www/js/rpg_core.js").exists() {
            RpgMakerVersion::MV
        } else {
            // 기본값은 MV
            RpgMakerVersion::MV
        };

        let metadata = Self::extract_metadata_from_system_json(&system_json_path)?;

        Ok(Some(GameProject::new(
            path.to_path_buf(),
            GameEngine::RpgMaker(version),
            version.to_string(),
            metadata,
        )))
    }

    /// 프로젝트 파일에서 메타데이터 추출
    fn extract_metadata_from_project_file(
        _content: &str,
        version: RpgMakerVersion,
    ) -> ProjectMetadata {
        // 프로젝트 파일은 단순 텍스트 (예: "RPGXP 1.02")
        ProjectMetadata::new()
            .with_title(format!("RPG Maker {} Project", version))
    }

    /// package.json에서 메타데이터 추출
    fn extract_metadata_from_package_json(json: &serde_json::Value) -> ProjectMetadata {
        let title = json
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let author = json
            .get("author")
            .and_then(|v| v.as_str())
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

    /// System.json에서 메타데이터 추출
    fn extract_metadata_from_system_json(path: &Path) -> Result<ProjectMetadata> {
        let content = fs::read_to_string(path)
            .map_err(|e| HayoTransError::FileReadError(e.to_string()))?;

        let json: serde_json::Value = serde_json::from_str(&content)?;

        let title = json
            .get("gameTitle")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let locale = json
            .get("locale")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(ProjectMetadata::new()
            .with_title(title.unwrap_or_else(|| "Unknown".to_string()))
            .with_language(locale.unwrap_or_else(|| "ja_JP".to_string())))
    }

    /// 프로젝트 파일 생성 (원래 C# 코드의 CreateProjectFile 함수)
    pub fn create_project_file(rgss_data_file: &Path, out_dir: &Path) -> Result<PathBuf> {
        let ext = rgss_data_file
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| format!(".{}", e))
            .ok_or_else(|| HayoTransError::UnknownEngine("No extension".to_string()))?;

        let version = RpgMakerVersion::from_extension(&ext)
            .ok_or_else(|| HayoTransError::UnknownEngine(format!("Unknown extension: {}", ext)))?;

        let project_filename = version.project_filename();
        let output_path = out_dir.join(project_filename);

        if let Some(content) = version.project_content() {
            fs::write(&output_path, content)
                .map_err(|e| HayoTransError::FileWriteError(e.to_string()))?;
            
            tracing::info!("Created project file: {:?}", output_path);
            Ok(output_path)
        } else {
            Err(HayoTransError::UnsupportedEngine(format!(
                "Cannot create project file for {}",
                version
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_file() {
        use std::env;
        
        let temp_dir = env::temp_dir();
        let test_file = temp_dir.join("test.rgssad");
        
        // 임시 파일 생성
        fs::write(&test_file, b"test").unwrap();
        
        let result = RpgMakerDetector::create_project_file(&test_file, &temp_dir);
        assert!(result.is_ok());
        
        let output_path = result.unwrap();
        assert!(output_path.exists());
        
        let content = fs::read_to_string(&output_path).unwrap();
        assert_eq!(content, "RPGXP 1.02");
        
        // 정리
        fs::remove_file(&test_file).ok();
        fs::remove_file(&output_path).ok();
    }
}
