use std::path::Path;
use std::fs;

use crate::types::{
    GameEngine, KiriKiriVersion, ProjectMetadata, GameProject, Result,
};

/// KiriKiri 프로젝트 감지기
pub struct KiriKiriDetector;

impl KiriKiriDetector {
    /// 디렉토리에서 KiriKiri 프로젝트 감지
    pub fn detect(path: &Path) -> Result<Option<GameProject>> {
        tracing::info!("Detecting KiriKiri project at: {:?}", path);

        // 1. .xp3 아카이브 파일 확인
        if !Self::has_xp3_archive(path)? {
            return Ok(None);
        }

        // 2. 실행 파일 확인
        let version = Self::detect_version(path)?;
        if version.is_none() {
            return Ok(None);
        }

        let version = version.unwrap();
        let metadata = Self::extract_metadata(path)?;

        Ok(Some(GameProject::new(
            path.to_path_buf(),
            GameEngine::KiriKiri(version),
            version.to_string(),
            metadata,
        )))
    }

    /// .xp3 아카이브 파일 존재 확인
    fn has_xp3_archive(path: &Path) -> Result<bool> {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(_) => return Ok(false),
        };

        for entry in entries.flatten() {
            let file_path = entry.path();
            if let Some(ext) = file_path.extension() {
                if ext == "xp3" {
                    tracing::info!("Found XP3 archive: {:?}", file_path);
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// KiriKiri 버전 감지
    fn detect_version(path: &Path) -> Result<Option<KiriKiriVersion>> {
        // krkrz.exe 확인 (KiriKiri Z)
        if path.join("krkrz.exe").exists() {
            tracing::info!("Found krkrz.exe - KiriKiri Z");
            return Ok(Some(KiriKiriVersion::Z));
        }

        // krkr.exe 확인 (KiriKiri KAG3)
        if path.join("krkr.exe").exists() {
            tracing::info!("Found krkr.exe - KiriKiri KAG3");
            return Ok(Some(KiriKiriVersion::KAG3));
        }

        // startup.tjs 확인
        if path.join("startup.tjs").exists() {
            tracing::info!("Found startup.tjs - assuming KiriKiri KAG3");
            return Ok(Some(KiriKiriVersion::KAG3));
        }

        Ok(None)
    }

    /// 메타데이터 추출
    fn extract_metadata(path: &Path) -> Result<ProjectMetadata> {
        let mut metadata = ProjectMetadata::new();

        // Config.tjs 파일에서 정보 추출 시도
        let config_path = path.join("Config.tjs");
        if config_path.exists() {
            if let Ok(content) = fs::read_to_string(&config_path) {
                // 간단한 파싱 (실제로는 더 복잡한 TJS 파서가 필요)
                if let Some(title) = Self::extract_title_from_config(&content) {
                    metadata = metadata.with_title(title);
                }
            }
        }

        // 디렉토리 이름을 기본 제목으로 사용
        if metadata.title.is_none() {
            if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                metadata = metadata.with_title(dir_name.to_string());
            }
        }

        Ok(metadata)
    }

    /// Config.tjs에서 제목 추출 (간단한 구현)
    fn extract_title_from_config(content: &str) -> Option<String> {
        // "title = "..." 패턴 찾기
        for line in content.lines() {
            if line.contains("title") && line.contains("=") {
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        return Some(line[start + 1..start + 1 + end].to_string());
                    }
                }
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_title_from_config() {
        let config = r#"
            title = "My Game";
            version = "1.0";
        "#;
        
        let title = KiriKiriDetector::extract_title_from_config(config);
        assert_eq!(title, Some("My Game".to_string()));
    }
}
