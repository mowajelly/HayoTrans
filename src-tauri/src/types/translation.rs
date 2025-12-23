use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// 번역 엔트리
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    /// 엔트리 ID
    pub id: String,
    /// 대사 ID
    pub dialogue_id: String,
    /// 원본 텍스트
    pub original_text: String,
    /// 번역된 텍스트
    pub translated_text: String,
    /// 번역기 타입
    pub translator: TranslatorType,
    /// 검토 상태
    pub status: ReviewStatus,
    /// 생성 시간
    pub created_at: DateTime<Utc>,
    /// 수정 시간
    pub updated_at: DateTime<Utc>,
    /// 검토자
    pub reviewed_by: Option<String>,
    /// 노트
    pub notes: Option<String>,
}

impl TranslationEntry {
    /// 새 번역 엔트리 생성
    pub fn new(
        dialogue_id: String,
        original_text: String,
        translated_text: String,
        translator: TranslatorType,
    ) -> Self {
        let now = Utc::now();
        let id = Self::generate_id(&dialogue_id, &now);
        
        Self {
            id,
            dialogue_id,
            original_text,
            translated_text,
            translator,
            status: ReviewStatus::Pending,
            created_at: now,
            updated_at: now,
            reviewed_by: None,
            notes: None,
        }
    }

    /// ID 생성
    fn generate_id(dialogue_id: &str, timestamp: &DateTime<Utc>) -> String {
        format!("trans_{}_{}", dialogue_id, timestamp.timestamp())
    }

    /// 번역 승인
    pub fn approve(&mut self, reviewer: impl Into<String>) {
        self.status = ReviewStatus::Approved;
        self.reviewed_by = Some(reviewer.into());
        self.updated_at = Utc::now();
    }

    /// 번역 거부
    pub fn reject(&mut self, reviewer: impl Into<String>, reason: impl Into<String>) {
        self.status = ReviewStatus::Rejected;
        self.reviewed_by = Some(reviewer.into());
        self.notes = Some(reason.into());
        self.updated_at = Utc::now();
    }

    /// 수정 필요 표시
    pub fn needs_revision(&mut self, reviewer: impl Into<String>, notes: impl Into<String>) {
        self.status = ReviewStatus::NeedsRevision;
        self.reviewed_by = Some(reviewer.into());
        self.notes = Some(notes.into());
        self.updated_at = Utc::now();
    }

    /// 번역 텍스트 업데이트
    pub fn update_translation(&mut self, new_text: String) {
        self.translated_text = new_text;
        self.updated_at = Utc::now();
    }
}

/// 번역기 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TranslatorType {
    /// Google Cloud Translation
    Gcp,
    /// ezTrans
    EzTrans,
    /// OpenAI API
    OpenAi,
    /// 인간 번역
    Human,
}

impl std::fmt::Display for TranslatorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Gcp => write!(f, "Google Cloud Translation"),
            Self::EzTrans => write!(f, "ezTrans"),
            Self::OpenAi => write!(f, "OpenAI"),
            Self::Human => write!(f, "Human"),
        }
    }
}

/// 검토 상태
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    /// 검토 대기
    Pending,
    /// 승인됨
    Approved,
    /// 수정 필요
    NeedsRevision,
    /// 거부됨
    Rejected,
}

impl std::fmt::Display for ReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending => write!(f, "Pending"),
            Self::Approved => write!(f, "Approved"),
            Self::NeedsRevision => write!(f, "Needs Revision"),
            Self::Rejected => write!(f, "Rejected"),
        }
    }
}

/// 번역 전략
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationStrategy {
    /// 기계번역만
    MachineOnly,
    /// AI 번역만
    AiOnly,
    /// 하이브리드
    Hybrid {
        /// 기계번역 먼저 수행
        machine_first: bool,
        /// AI 검토
        ai_review: bool,
    },
}

impl Default for TranslationStrategy {
    fn default() -> Self {
        Self::Hybrid {
            machine_first: true,
            ai_review: false,
        }
    }
}

/// 스토리 컨텍스트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoryContext {
    /// 스토리 요약
    pub summary: String,
    /// 캐릭터 목록
    pub characters: Vec<Character>,
    /// 현재 챕터
    pub current_chapter: Option<String>,
}

/// 캐릭터 정보
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    /// 캐릭터 이름
    pub name: String,
    /// 캐릭터 설명
    pub description: Option<String>,
    /// 말투 특징
    pub speech_style: Option<String>,
}

/// 맵 컨텍스트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapContext {
    /// 맵 이름
    pub map_name: String,
    /// 장소 타입
    pub location_type: LocationType,
    /// 분위기
    pub atmosphere: Option<String>,
}

/// 장소 타입
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationType {
    /// 마을
    Town,
    /// 던전
    Dungeon,
    /// 필드
    Field,
    /// 실내
    Indoor,
    /// 기타
    Other,
}

impl std::fmt::Display for LocationType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Town => write!(f, "Town"),
            Self::Dungeon => write!(f, "Dungeon"),
            Self::Field => write!(f, "Field"),
            Self::Indoor => write!(f, "Indoor"),
            Self::Other => write!(f, "Other"),
        }
    }
}

/// 번역 요청
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    /// 번역할 텍스트
    pub text: String,
    /// 원본 언어
    pub source_language: String,
    /// 대상 언어
    pub target_language: String,
    /// 컨텍스트 (선택사항)
    pub context: Option<String>,
}

/// 번역 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationResponse {
    /// 번역된 텍스트
    pub translated_text: String,
    /// 번역기 타입
    pub translator: TranslatorType,
    /// 신뢰도 (0.0 ~ 1.0)
    pub confidence: Option<f32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translation_entry_creation() {
        let entry = TranslationEntry::new(
            "dialogue_1".to_string(),
            "こんにちは".to_string(),
            "Hello".to_string(),
            TranslatorType::Gcp,
        );

        assert_eq!(entry.dialogue_id, "dialogue_1");
        assert_eq!(entry.status, ReviewStatus::Pending);
        assert!(entry.reviewed_by.is_none());
    }

    #[test]
    fn test_translation_entry_approve() {
        let mut entry = TranslationEntry::new(
            "dialogue_1".to_string(),
            "こんにちは".to_string(),
            "Hello".to_string(),
            TranslatorType::Gcp,
        );

        entry.approve("reviewer1");
        assert_eq!(entry.status, ReviewStatus::Approved);
        assert_eq!(entry.reviewed_by, Some("reviewer1".to_string()));
    }

    #[test]
    fn test_translation_entry_reject() {
        let mut entry = TranslationEntry::new(
            "dialogue_1".to_string(),
            "こんにちは".to_string(),
            "Hello".to_string(),
            TranslatorType::Gcp,
        );

        entry.reject("reviewer1", "Incorrect translation");
        assert_eq!(entry.status, ReviewStatus::Rejected);
        assert_eq!(entry.reviewed_by, Some("reviewer1".to_string()));
        assert_eq!(entry.notes, Some("Incorrect translation".to_string()));
    }

    #[test]
    fn test_translator_type_display() {
        assert_eq!(TranslatorType::Gcp.to_string(), "Google Cloud Translation");
        assert_eq!(TranslatorType::EzTrans.to_string(), "ezTrans");
        assert_eq!(TranslatorType::OpenAi.to_string(), "OpenAI");
        assert_eq!(TranslatorType::Human.to_string(), "Human");
    }
}
