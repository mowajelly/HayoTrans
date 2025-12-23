use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::engine::GameEngine;

/// 게임 프로젝트 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameProject {
    /// 프로젝트 고유 ID
    pub id: String,
    /// 프로젝트 경로
    pub path: PathBuf,
    /// 게임 엔진
    pub engine: GameEngine,
    /// 엔진 버전 문자열
    pub version: String,
    /// 프로젝트 메타데이터
    pub metadata: ProjectMetadata,
}

impl GameProject {
    /// 새 프로젝트 생성
    pub fn new(
        path: PathBuf,
        engine: GameEngine,
        version: String,
        metadata: ProjectMetadata,
    ) -> Self {
        let id = Self::generate_id(&path);
        Self {
            id,
            path,
            engine,
            version,
            metadata,
        }
    }

    /// 경로로부터 프로젝트 ID 생성
    fn generate_id(path: &PathBuf) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        path.hash(&mut hasher);
        format!("project_{:x}", hasher.finish())
    }

    /// 프로젝트가 유효한지 확인
    pub fn is_valid(&self) -> bool {
        self.path.exists() && self.engine.is_supported()
    }

    /// 프로젝트 이름 반환
    pub fn name(&self) -> String {
        self.metadata
            .title
            .clone()
            .unwrap_or_else(|| {
                self.path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown")
                    .to_string()
            })
    }
}

/// 프로젝트 메타데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// 게임 제목
    pub title: Option<String>,
    /// 제작자
    pub author: Option<String>,
    /// 원본 언어
    pub language: Option<String>,
    /// 인코딩
    pub encoding: Option<String>,
    /// 게임 설명
    pub description: Option<String>,
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            title: None,
            author: None,
            language: None,
            encoding: Some("UTF-8".to_string()),
            description: None,
        }
    }
}

impl ProjectMetadata {
    /// 새 메타데이터 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 제목 설정
    pub fn with_title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// 제작자 설정
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// 언어 설정
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// 인코딩 설정
    pub fn with_encoding(mut self, encoding: impl Into<String>) -> Self {
        self.encoding = Some(encoding.into());
        self
    }

    /// 설명 설정
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// 프로젝트 감지 결과
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    /// 감지 성공 여부
    pub success: bool,
    /// 감지된 프로젝트 (성공 시)
    pub project: Option<GameProject>,
    /// 에러 메시지 (실패 시)
    pub error: Option<String>,
    /// 감지 과정에서 발견된 정보
    pub details: Vec<String>,
}

impl DetectionResult {
    /// 성공 결과 생성
    pub fn success(project: GameProject) -> Self {
        Self {
            success: true,
            project: Some(project),
            error: None,
            details: vec![],
        }
    }

    /// 실패 결과 생성
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            project: None,
            error: Some(error.into()),
            details: vec![],
        }
    }

    /// 상세 정보 추가
    pub fn with_details(mut self, details: Vec<String>) -> Self {
        self.details = details;
        self
    }

    /// 상세 정보 추가 (단일)
    pub fn add_detail(mut self, detail: impl Into<String>) -> Self {
        self.details.push(detail.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::engine::{GameEngine, RpgMakerVersion};

    #[test]
    fn test_project_metadata_builder() {
        let metadata = ProjectMetadata::new()
            .with_title("Test Game")
            .with_author("Test Author")
            .with_language("Japanese");

        assert_eq!(metadata.title, Some("Test Game".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert_eq!(metadata.language, Some("Japanese".to_string()));
    }

    #[test]
    fn test_game_project_name() {
        let metadata = ProjectMetadata::new().with_title("My Game");
        let project = GameProject::new(
            PathBuf::from("/test/path"),
            GameEngine::RpgMaker(RpgMakerVersion::MV),
            "1.0.0".to_string(),
            metadata,
        );

        assert_eq!(project.name(), "My Game");
    }

    #[test]
    fn test_detection_result() {
        let success = DetectionResult::success(GameProject::new(
            PathBuf::from("/test"),
            GameEngine::RpgMaker(RpgMakerVersion::MV),
            "1.0.0".to_string(),
            ProjectMetadata::new(),
        ));

        assert!(success.success);
        assert!(success.project.is_some());
        assert!(success.error.is_none());

        let failure = DetectionResult::failure("Test error");
        assert!(!failure.success);
        assert!(failure.project.is_none());
        assert_eq!(failure.error, Some("Test error".to_string()));
    }
}
