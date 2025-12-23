use serde::{Deserialize, Serialize};
use std::fmt;

/// HayoTrans 에러 타입
#[derive(Debug, thiserror::Error)]
pub enum HayoTransError {
    /// 게임 엔진을 감지할 수 없음
    #[error("Unknown game engine: {0}")]
    UnknownEngine(String),

    /// 지원하지 않는 게임 엔진
    #[error("Unsupported game engine: {0}")]
    UnsupportedEngine(String),

    /// 파일을 찾을 수 없음
    #[error("File not found: {0}")]
    FileNotFound(String),

    /// 디렉토리를 찾을 수 없음
    #[error("Directory not found: {0}")]
    DirectoryNotFound(String),

    /// 파일 읽기 오류
    #[error("Failed to read file: {0}")]
    FileReadError(String),

    /// 파일 쓰기 오류
    #[error("Failed to write file: {0}")]
    FileWriteError(String),

    /// 파싱 오류
    #[error("Parse error: {0}")]
    ParseError(String),

    /// JSON 파싱 오류
    #[error("JSON parse error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// IO 오류
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// 번역 오류
    #[error("Translation error: {0}")]
    TranslationError(String),

    /// API 오류
    #[error("API error: {0}")]
    ApiError(String),

    /// 데이터베이스 오류
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// 인코딩 오류
    #[error("Encoding error: {0}")]
    EncodingError(String),

    /// 압축 해제 오류
    #[error("Decompression error: {0}")]
    DecompressionError(String),

    /// 압축 오류
    #[error("Compression error: {0}")]
    CompressionError(String),

    /// 검증 오류
    #[error("Validation error: {0}")]
    ValidationError(String),

    /// 설정 오류
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// 기타 오류
    #[error("Error: {0}")]
    Other(String),
}

/// 결과 타입 별칭
pub type Result<T> = std::result::Result<T, HayoTransError>;

/// 프론트엔드로 전달할 에러 응답
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub details: Option<String>,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: None,
        }
    }

    pub fn with_details(error: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            details: Some(details.into()),
        }
    }
}

impl From<HayoTransError> for ErrorResponse {
    fn from(err: HayoTransError) -> Self {
        Self {
            error: err.to_string(),
            details: None,
        }
    }
}

impl fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(details) = &self.details {
            write!(f, "{}: {}", self.error, details)
        } else {
            write!(f, "{}", self.error)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HayoTransError::UnknownEngine("Test".to_string());
        assert_eq!(err.to_string(), "Unknown game engine: Test");
    }

    #[test]
    fn test_error_response() {
        let response = ErrorResponse::new("Test error");
        assert_eq!(response.error, "Test error");
        assert!(response.details.is_none());

        let response = ErrorResponse::with_details("Test error", "More details");
        assert_eq!(response.error, "Test error");
        assert_eq!(response.details, Some("More details".to_string()));
    }
}
