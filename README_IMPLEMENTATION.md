# HayoTrans 구현 현황
## 완료된 작업
### 1. 프로젝트 아키텍처 설계 ✅
- 5대 핵심 모듈 구조 설계 완료
  - Retriever (게임 엔진 감지)
  - Parser (게임 데이터 파싱)
  - Translator (번역 엔진)
  - Cache (번역 캐시 관리)
  - Repack (게임 재패키징)
### 2. 타입 시스템 구현 ✅
**위치**: `src-tauri/src/types/`
- [`engine.rs`](src-tauri/src/types/engine.rs) - 게임 엔진 타입 정의
  - `RpgMakerVersion` (XP, VX, VXAce, MV, MZ)
  - `KiriKiriVersion` (KAG3, Z)
  - `V8Engine` (NwJs, Electron, Generic)
  - `GameEngine` enum
- [`error.rs`](src-tauri/src/types/error.rs) - 에러 처리
  - `HayoTransError` - 모든 에러 타입
  - `ErrorResponse` - 프론트엔드 응답용
- [`project.rs`](src-tauri/src/types/project.rs) - 프로젝트 정보
  - `GameProject` - 게임 프로젝트 정보
  - `ProjectMetadata` - 메타데이터
  - `DetectionResult` - 감지 결과
- [`dialogue.rs`](src-tauri/src/types/dialogue.rs) - 대사 관련
  - `DialogueLine` - 대사 라인
  - `DialogueContext` - 대사 컨텍스트
  - `EventData`, `PluginData`, `GameFile`
- [`translation.rs`](src-tauri/src/types/translation.rs) - 번역 관련
  - `TranslationEntry` - 번역 엔트리
  - `TranslatorType` - 번역기 타입
  - `ReviewStatus` - 검토 상태
  - `TranslationStrategy` - 번역 전략
  - `StoryContext`, `MapContext` - AI 번역용 컨텍스트
### 3. Retriever 모듈 구현 ✅
**위치**: `src-tauri/src/retriever/`
- [`rpg_maker.rs`](src-tauri/src/retriever/rpg_maker.rs) - RPG Maker 감지
  - 프로젝트 파일 감지 (.rxproj, .rvproj, .rvproj2)
  - 데이터 아카이브 감지 (.rgssad, .rgss2a, .rgss3a)
  - package.json 감지 (MV/MZ)
  - www/data 디렉토리 감지
  - **`create_project_file()` - 원본 C# 코드의 Rust 구현** ✨
- [`kirikiri.rs`](src-tauri/src/retriever/kirikiri.rs) - KiriKiri 감지
  - .xp3 아카이브 감지
  - 실행 파일 감지 (krkr.exe, krkrz.exe)
  - Config.tjs 파싱
- [`v8_engine.rs`](src-tauri/src/retriever/v8_engine.rs) - V8 엔진 감지
  - NW.js 감지
  - Electron 감지
  - package.json 분석
- [`detector.rs`](src-tauri/src/retriever/detector.rs) - 통합 감지기
  - 모든 엔진 자동 감지
  - 일괄 처리 지원
### 4. Tauri Commands 구현 ✅
**위치**: `src-tauri/src/commands/`
- [`retriever.rs`](src-tauri/src/commands/retriever.rs)
  - `detect_game_engine()` - 게임 엔진 감지
  - `is_game_supported()` - 지원 여부 확인
  - `create_rpg_maker_project_file()` - RPG Maker 프로젝트 파일 생성
### 5. 의존성 설정 ✅
**위치**: [`src-tauri/Cargo.toml`](src-tauri/Cargo.toml)
추가된 crates:
- `thiserror`, `anyhow` - 에러 처리
- `tokio`, `async-trait` - 비동기 처리
- `reqwest` - HTTP 클라이언트 (번역 API용)
- `chrono` - 날짜/시간
- `zip`, `flate2` - 압축/아카이브
- `encoding_rs` - 인코딩
- `tracing`, `tracing-subscriber` - 로깅
## 원본 C# 코드 구현 상태
### CreateProjectFile 함수 ✅
**원본 위치**: 제공하신 C# 코드
**Rust 구현**: [`src-tauri/src/retriever/rpg_maker.rs:220`](src-tauri/src/retriever/rpg_maker.rs:220)
```rust
pub fn create_project_file(rgss_data_file: &Path, out_dir: &Path) -> Result<PathBuf>
```
**기능**:
- `.rgssad` → `Game.rxproj` (RPGXP 1.02)
- `.rgss2a` → `Game.rvproj` (RPGVX 1.02)
- `.rgss3a` → `Game.rvproj2` (RPGVXAce 1.00)
**개선사항**:
- Rust의 타입 안전성 활용
- 에러 처리 강화
- 크로스 플랫폼 경로 처리
- 로깅 추가
## 프로젝트 구조
```
HayoTrans/
├── plans/
│   ├── hayotrans-architecture.md      # 전체 아키텍처 문서
│   └── rpg-maker-implementation.md    # 초기 RPG Maker 계획
├── src-tauri/
│   ├── Cargo.toml                     # Rust 의존성
│   └── src/
│       ├── main.rs                    # 진입점
│       ├── lib.rs                     # Tauri 앱 설정
│       ├── types/                     # 타입 정의 ✅
│       │   ├── mod.rs
│       │   ├── engine.rs
│       │   ├── error.rs
│       │   ├── project.rs
│       │   ├── dialogue.rs
│       │   └── translation.rs
│       ├── retriever/                 # 게임 엔진 감지 ✅
│       │   ├── mod.rs
│       │   ├── detector.rs
│       │   ├── rpg_maker.rs
│       │   ├── kirikiri.rs
│       │   └── v8_engine.rs
│       ├── commands/                  # Tauri commands ✅
│       │   ├── mod.rs
│       │   └── retriever.rs
│       ├── parser/                    # 🚧 다음 단계
│       ├── translator/                # 🚧 다음 단계
│       ├── cache/                     # 🚧 다음 단계
│       └── repack/                    # 🚧 다음 단계
└── src/                               # 프론트엔드 (SolidJS)
    ├── App.tsx
    └── ...
```
## 다음 단계
### Phase 2: Parser 모듈 (우선순위 높음)
1. RPG Maker MV/MZ JSON 파서
2. RPG Maker XP/VX/VXAce Marshal 파서
3. 대사 추출 로직
4. 이벤트 파싱
### Phase 3: Cache 모듈
1. SQLite 데이터베이스 설정
2. 번역 CRUD 작업
3. 검토 워크플로우
### Phase 4: Translator 모듈
1. GCP Translation API 연동
2. ezTrans 연동
3. OpenAI API 연동
### Phase 5: Repack 모듈
1. RPG Maker 재패키징
2. 백업 시스템
### Phase 6: 프론트엔드 UI
1. 프로젝트 선택 UI
2. 번역 편집기
3. 검토 패널
## 빌드 요구사항
### Windows
- Visual Studio 2017 이상 또는 Build Tools for Visual Studio
- Visual C++ 빌드 도구 필요
### 빌드 명령어
```bash
# Rust 코드 체크
cd src-tauri
cargo check
# 전체 빌드
cargo build
# Tauri 앱 실행
cd ..
pnpm tauri dev
```
## 테스트
각 모듈에 단위 테스트가 포함되어 있습니다:
```bash
cd src-tauri
cargo test
```
## 사용 예시
### 프론트엔드에서 사용
```typescript
import { invoke } from '@tauri-apps/api/core';
// 게임 엔진 감지
const result = await invoke('detect_game_engine', {
  path: 'C:/Games/MyRPGGame'
});
// RPG Maker 프로젝트 파일 생성
const projectFile = await invoke('create_rpg_maker_project_file', {
  rgssFile: 'C:/Games/MyGame/Game.rgssad',
  outputDir: 'C:/Games/MyGame'
});
```
## 주요 기능
### 지원하는 게임 엔진
- ✅ RPG Maker XP
- ✅ RPG Maker VX
- ✅ RPG Maker VX Ace
- ✅ RPG Maker MV
- ✅ RPG Maker MZ
- ✅ KiriKiri (기본 지원)
- ✅ NW.js 기반 게임
- ✅ Electron 기반 게임
### 감지 방법
1. **파일 시그니처**: 프로젝트 파일, 아카이브 파일
2. **디렉토리 구조**: www/data, resources 등
3. **메타데이터**: package.json, System.json
4. **실행 파일**: .exe 파일 확인
## 문서
- [`plans/hayotrans-architecture.md`](plans/hayotrans-architecture.md) - 전체 시스템 아키텍처
- [`plans/rpg-maker-implementation.md`](plans/rpg-maker-implementation.md) - 초기 구현 계획
- 이 파일 - 구현 현황 및 가이드
## 기여 가이드
1. 새로운 게임 엔진 지원 추가 시:
   - `src-tauri/src/types/engine.rs`에 enum 추가
   - `src-tauri/src/retriever/`에 감지기 구현
   - `src-tauri/src/retriever/detector.rs`에 통합
2. 새로운 기능 추가 시:
   - 해당 모듈에 구현
   - `src-tauri/src/commands/`에 Tauri command 추가
   - `src-tauri/src/lib.rs`에 command 등록
## 라이선스
MIT License