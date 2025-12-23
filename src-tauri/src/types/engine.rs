use serde::{Deserialize, Serialize};

/// RPG Maker 버전
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RpgMakerVersion {
    /// RPG Maker XP (.rgssad, Game.rxproj)
    XP,
    /// RPG Maker VX (.rgss2a, Game.rvproj)
    VX,
    /// RPG Maker VX Ace (.rgss3a, Game.rvproj2)
    VXAce,
    /// RPG Maker MV (www/data/, package.json with "rpg_core")
    MV,
    /// RPG Maker MZ (www/data/, package.json with "rmmz_core")
    MZ,
}

impl RpgMakerVersion {
    /// 파일 확장자로부터 버전 감지
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            ".rgssad" => Some(Self::XP),
            ".rgss2a" => Some(Self::VX),
            ".rgss3a" => Some(Self::VXAce),
            _ => None,
        }
    }

    /// 프로젝트 파일명 반환
    pub fn project_filename(&self) -> &str {
        match self {
            Self::XP => "Game.rxproj",
            Self::VX => "Game.rvproj",
            Self::VXAce => "Game.rvproj2",
            Self::MV | Self::MZ => "package.json",
        }
    }

    /// 프로젝트 파일 내용 반환 (XP/VX/VXAce만 해당)
    pub fn project_content(&self) -> Option<&str> {
        match self {
            Self::XP => Some("RPGXP 1.02"),
            Self::VX => Some("RPGVX 1.02"),
            Self::VXAce => Some("RPGVXAce 1.00"),
            Self::MV | Self::MZ => None,
        }
    }

    /// 데이터 디렉토리 경로
    pub fn data_directory(&self) -> &str {
        match self {
            Self::XP | Self::VX | Self::VXAce => "Data",
            Self::MV | Self::MZ => "www/data",
        }
    }

    /// Marshal 포맷 사용 여부
    pub fn uses_marshal(&self) -> bool {
        matches!(self, Self::XP | Self::VX | Self::VXAce)
    }

    /// JSON 포맷 사용 여부
    pub fn uses_json(&self) -> bool {
        matches!(self, Self::MV | Self::MZ)
    }
}

impl std::fmt::Display for RpgMakerVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XP => write!(f, "RPG Maker XP"),
            Self::VX => write!(f, "RPG Maker VX"),
            Self::VXAce => write!(f, "RPG Maker VX Ace"),
            Self::MV => write!(f, "RPG Maker MV"),
            Self::MZ => write!(f, "RPG Maker MZ"),
        }
    }
}

/// KiriKiri 버전
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KiriKiriVersion {
    /// KiriKiri KAG3 (.xp3 archives)
    KAG3,
    /// KiriKiri Z (최신 버전)
    Z,
}

impl std::fmt::Display for KiriKiriVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KAG3 => write!(f, "KiriKiri KAG3"),
            Self::Z => write!(f, "KiriKiri Z"),
        }
    }
}

/// V8 기반 엔진
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum V8Engine {
    /// NW.js (package.json with "nw")
    NwJs,
    /// Electron (package.json with "electron")
    Electron,
    /// 기타 V8 엔진
    Generic,
}

impl std::fmt::Display for V8Engine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NwJs => write!(f, "NW.js"),
            Self::Electron => write!(f, "Electron"),
            Self::Generic => write!(f, "Generic V8 Engine"),
        }
    }
}

/// 게임 엔진 타입
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum GameEngine {
    /// RPG Maker
    RpgMaker(RpgMakerVersion),
    /// KiriKiri
    KiriKiri(KiriKiriVersion),
    /// V8 기반 엔진
    V8Engine(V8Engine),
    /// 알 수 없는 엔진
    Unknown,
}

impl GameEngine {
    /// 엔진 이름 반환
    pub fn name(&self) -> String {
        match self {
            Self::RpgMaker(v) => v.to_string(),
            Self::KiriKiri(v) => v.to_string(),
            Self::V8Engine(v) => v.to_string(),
            Self::Unknown => "Unknown Engine".to_string(),
        }
    }

    /// 엔진이 지원되는지 확인
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

impl std::fmt::Display for GameEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rpg_maker_version_from_extension() {
        assert_eq!(
            RpgMakerVersion::from_extension(".rgssad"),
            Some(RpgMakerVersion::XP)
        );
        assert_eq!(
            RpgMakerVersion::from_extension(".rgss2a"),
            Some(RpgMakerVersion::VX)
        );
        assert_eq!(
            RpgMakerVersion::from_extension(".rgss3a"),
            Some(RpgMakerVersion::VXAce)
        );
        assert_eq!(RpgMakerVersion::from_extension(".unknown"), None);
    }

    #[test]
    fn test_rpg_maker_version_project_filename() {
        assert_eq!(RpgMakerVersion::XP.project_filename(), "Game.rxproj");
        assert_eq!(RpgMakerVersion::VX.project_filename(), "Game.rvproj");
        assert_eq!(RpgMakerVersion::VXAce.project_filename(), "Game.rvproj2");
        assert_eq!(RpgMakerVersion::MV.project_filename(), "package.json");
        assert_eq!(RpgMakerVersion::MZ.project_filename(), "package.json");
    }

    #[test]
    fn test_rpg_maker_version_uses_marshal() {
        assert!(RpgMakerVersion::XP.uses_marshal());
        assert!(RpgMakerVersion::VX.uses_marshal());
        assert!(RpgMakerVersion::VXAce.uses_marshal());
        assert!(!RpgMakerVersion::MV.uses_marshal());
        assert!(!RpgMakerVersion::MZ.uses_marshal());
    }

    #[test]
    fn test_game_engine_is_supported() {
        assert!(GameEngine::RpgMaker(RpgMakerVersion::MV).is_supported());
        assert!(GameEngine::KiriKiri(KiriKiriVersion::Z).is_supported());
        assert!(!GameEngine::Unknown.is_supported());
    }
}
