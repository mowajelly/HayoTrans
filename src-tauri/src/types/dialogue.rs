use serde::{Deserialize, Serialize};

/// 대사 라인
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueLine {
    /// 대사 고유 ID
    pub id: String,
    /// 파일 경로
    pub file: String,
    /// 라인 번호
    pub line_number: usize,
    /// 화자 (있는 경우)
    pub speaker: Option<String>,
    /// 원본 텍스트
    pub original_text: String,
    /// 대사 컨텍스트
    pub context: DialogueContext,
}

impl DialogueLine {
    /// 새 대사 라인 생성
    pub fn new(
        id: String,
        file: String,
        line_number: usize,
        original_text: String,
    ) -> Self {
        Self {
            id,
            file,
            line_number,
            speaker: None,
            original_text,
            context: DialogueContext::default(),
        }
    }

    /// 화자 설정
    pub fn with_speaker(mut self, speaker: impl Into<String>) -> Self {
        self.speaker = Some(speaker.into());
        self
    }

    /// 컨텍스트 설정
    pub fn with_context(mut self, context: DialogueContext) -> Self {
        self.context = context;
        self
    }

    /// 대사가 비어있는지 확인
    pub fn is_empty(&self) -> bool {
        self.original_text.trim().is_empty()
    }

    /// 번역이 필요한지 확인 (일본어/한국어/중국어 문자 포함 여부)
    pub fn needs_translation(&self) -> bool {
        self.original_text.chars().any(|c| {
            // 일본어 (히라가나, 가타카나, 한자)
            ('\u{3040}'..='\u{309F}').contains(&c) ||  // 히라가나
            ('\u{30A0}'..='\u{30FF}').contains(&c) ||  // 가타카나
            // 한국어 (한글)
            ('\u{AC00}'..='\u{D7AF}').contains(&c) ||  // 한글 음절
            // 중국어 (한자)
            ('\u{4E00}'..='\u{9FFF}').contains(&c)     // CJK 통합 한자
        })
    }
}

/// 대사 컨텍스트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueContext {
    /// 맵 이름
    pub map_name: Option<String>,
    /// 이벤트 이름
    pub event_name: Option<String>,
    /// 이전 대사들
    pub preceding_lines: Vec<String>,
    /// 태그들
    pub tags: Vec<String>,
}

impl Default for DialogueContext {
    fn default() -> Self {
        Self {
            map_name: None,
            event_name: None,
            preceding_lines: Vec::new(),
            tags: Vec::new(),
        }
    }
}

impl DialogueContext {
    /// 새 컨텍스트 생성
    pub fn new() -> Self {
        Self::default()
    }

    /// 맵 이름 설정
    pub fn with_map_name(mut self, map_name: impl Into<String>) -> Self {
        self.map_name = Some(map_name.into());
        self
    }

    /// 이벤트 이름 설정
    pub fn with_event_name(mut self, event_name: impl Into<String>) -> Self {
        self.event_name = Some(event_name.into());
        self
    }

    /// 이전 대사 추가
    pub fn add_preceding_line(mut self, line: impl Into<String>) -> Self {
        self.preceding_lines.push(line.into());
        self
    }

    /// 태그 추가
    pub fn add_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
}

/// 이벤트 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventData {
    /// 이벤트 ID
    pub id: String,
    /// 이벤트 이름
    pub name: String,
    /// 이벤트 커맨드들
    pub commands: Vec<EventCommand>,
}

/// 이벤트 커맨드
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCommand {
    /// 커맨드 코드
    pub code: i32,
    /// 커맨드 파라미터
    pub parameters: Vec<serde_json::Value>,
}

/// 플러그인 데이터
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginData {
    /// 플러그인 이름
    pub name: String,
    /// 플러그인 버전
    pub version: String,
    /// 플러그인 파라미터
    pub parameters: serde_json::Value,
}

/// 게임 파일 (언팩된 파일)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameFile {
    /// 파일 경로 (상대 경로)
    pub path: String,
    /// 파일 내용
    pub content: Vec<u8>,
    /// 파일 크기
    pub size: usize,
}

impl GameFile {
    /// 새 게임 파일 생성
    pub fn new(path: String, content: Vec<u8>) -> Self {
        let size = content.len();
        Self {
            path,
            content,
            size,
        }
    }

    /// 텍스트로 변환 (UTF-8)
    pub fn as_text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.content.clone())
    }

    /// 파일 확장자 반환
    pub fn extension(&self) -> Option<&str> {
        std::path::Path::new(&self.path)
            .extension()
            .and_then(|e| e.to_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dialogue_line_needs_translation() {
        let japanese = DialogueLine::new(
            "1".to_string(),
            "test.json".to_string(),
            1,
            "こんにちは".to_string(),
        );
        assert!(japanese.needs_translation());

        let english = DialogueLine::new(
            "2".to_string(),
            "test.json".to_string(),
            2,
            "Hello".to_string(),
        );
        assert!(!english.needs_translation());

        let korean = DialogueLine::new(
            "3".to_string(),
            "test.json".to_string(),
            3,
            "안녕하세요".to_string(),
        );
        assert!(korean.needs_translation());
    }

    #[test]
    fn test_dialogue_context_builder() {
        let context = DialogueContext::new()
            .with_map_name("Town")
            .with_event_name("NPC Dialogue")
            .add_preceding_line("Hello")
            .add_tag("greeting");

        assert_eq!(context.map_name, Some("Town".to_string()));
        assert_eq!(context.event_name, Some("NPC Dialogue".to_string()));
        assert_eq!(context.preceding_lines.len(), 1);
        assert_eq!(context.tags.len(), 1);
    }

    #[test]
    fn test_game_file() {
        let content = b"test content".to_vec();
        let file = GameFile::new("test.txt".to_string(), content.clone());

        assert_eq!(file.size, content.len());
        assert_eq!(file.extension(), Some("txt"));
        assert_eq!(file.as_text().unwrap(), "test content");
    }
}
