# Coding Convention & AI Collaboration Guide

> Rust/Ratatui 프로젝트에 맞는 코딩 컨벤션 및 AI 협업 가이드입니다.

---

## MVP 캡슐

| # | 항목 | 내용 |
|---|------|------|
| 1 | 목표 | Claude Code 오케스트레이션 진행 상황을 터미널에서 실시간 시각화 |
| 2 | 페르소나 | Claude Code 헤비 유저 |
| 3 | 핵심 기능 | FEAT-1: Watch 모드, FEAT-2: 간트차트, FEAT-3: 에러 AI 분석 |
| 4 | 성공 지표 (노스스타) | GitHub Stars 100+ |
| 5 | 입력 지표 | 주간 다운로드 수 |
| 6 | 비기능 요구 | 렌더링 60fps |
| 7 | Out-of-scope | Wrapper 모드, 비용 트래커 |
| 8 | Top 리스크 | Rust 개발 속도 |
| 9 | 완화/실험 | 프로토타입 먼저 |
| 10 | 다음 단계 | Ratatui 프로토타입 |

---

## 1. 핵심 원칙

### 1.1 Rust 정신 따르기

- **소유권을 존중하라**: 불필요한 `.clone()` 지양, 참조로 해결 가능하면 참조 사용
- **에러를 무시하지 마라**: `.unwrap()` 금지 (테스트 코드 제외), `?` 연산자 또는 명시적 에러 처리
- **컴파일러를 믿어라**: `clippy` 경고를 모두 해결, `#[allow(unused)]` 남용 금지

### 1.2 신뢰하되, 검증하라

AI가 생성한 코드는 반드시 검증:

- [ ] `cargo clippy` 경고 0개
- [ ] `cargo test` 전체 통과
- [ ] `cargo fmt` 통과
- [ ] unsafe 코드 없음 (특별한 이유가 없는 한)

---

## 2. 프로젝트 구조

### 2.1 디렉토리 구조

```
oh-my-claude-board/
├── Cargo.toml              # 프로젝트 메타데이터 + 의존성
├── Cargo.lock
├── src/
│   ├── main.rs             # CLI 엔트리포인트 (clap)
│   ├── app.rs              # App 구조체 + 이벤트 루프
│   ├── lib.rs              # 라이브러리 루트 (테스트용 공개 인터페이스)
│   ├── ui/                 # TUI 렌더링 레이어
│   │   ├── mod.rs
│   │   ├── layout.rs       # 화면 분할 레이아웃
│   │   ├── gantt.rs        # 간트차트 위젯
│   │   ├── detail.rs       # 태스크 상세 패널
│   │   ├── claude_output.rs # Claude Code 출력 패널
│   │   ├── statusbar.rs    # 하단 상태바
│   │   └── help.rs         # 도움말 오버레이
│   ├── data/               # 데이터 처리 레이어
│   │   ├── mod.rs
│   │   ├── tasks_parser.rs # TASKS.md 파서
│   │   ├── hook_parser.rs  # Hook 이벤트 파서
│   │   ├── state.rs        # Unified State Model
│   │   └── watcher.rs      # 파일 감시 (notify)
│   ├── analysis/           # 에러 분석 레이어
│   │   ├── mod.rs
│   │   ├── rules.rs        # 규칙 기반 에러 분석
│   │   └── api.rs          # Anthropic API (선택)
│   ├── event.rs            # 키보드/파일/타이머 이벤트 통합
│   └── config.rs           # 설정 (TOML, clap)
├── tests/
│   ├── parser_test.rs      # 파서 통합 테스트
│   ├── state_test.rs       # 상태 모델 테스트
│   └── fixtures/           # 테스트 데이터
│       ├── sample_tasks.md
│       └── sample_hooks/
├── benches/                # 벤치마크
│   └── parser_bench.rs
└── docs/
    └── planning/           # 기획 문서
```

### 2.2 네이밍 규칙

| 대상 | 규칙 | 예시 |
|------|------|------|
| 파일명 | snake_case | `tasks_parser.rs` |
| 모듈명 | snake_case | `mod data` |
| 구조체/열거형 | PascalCase | `DashboardState`, `TaskStatus` |
| 함수/메서드 | snake_case | `parse_tasks_md()` |
| 상수 | UPPER_SNAKE_CASE | `MAX_LOG_ENTRIES` |
| 타입 별칭 | PascalCase | `type TaskId = String` |
| 트레이트 | PascalCase + 형용사/동사 | `Parsable`, `Renderable` |
| 라이프타임 | 짧은 소문자 | `'a`, `'src` |
| 제네릭 | 단일 대문자 | `T`, `E` |

---

## 3. 아키텍처 원칙

### 3.1 레이어 분리

```
┌─────────────────────┐
│   UI Layer (ui/)    │  Ratatui 위젯 + 렌더링
├─────────────────────┤
│   App Layer (app)   │  이벤트 루프 + 상태 관리
├─────────────────────┤
│  Data Layer (data/) │  파싱 + 파일 감시
├─────────────────────┤
│ Analysis (analysis/)│  에러 분석
└─────────────────────┘
```

**의존 방향**: UI → App → Data/Analysis (역방향 금지)

### 3.2 모듈 크기 가이드

- 한 파일: 300줄 이하 권장 (500줄 초과 시 분리)
- 한 함수: 50줄 이하 권장
- 한 구조체: 필드 10개 이하

### 3.3 에러 처리 전략

```rust
// 라이브러리 에러: thiserror
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("TASKS.md 파싱 실패: {0}")]
    TasksParse(String),
    #[error("Hook 이벤트 파싱 실패: {0}")]
    HookParse(String),
    #[error("파일 읽기 실패: {path}")]
    FileRead { path: PathBuf, source: io::Error },
}

// 애플리케이션 에러: anyhow
fn main() -> anyhow::Result<()> {
    // ... anyhow::Context로 에러 체이닝
}
```

---

## 4. Rust 코딩 스타일

### 4.1 필수 규칙

| 규칙 | 설명 |
|------|------|
| `.unwrap()` 금지 | 테스트 코드 제외, 프로덕션에서는 `?` 또는 `expect("이유")` |
| `unsafe` 금지 | 특별한 사유 + 코멘트 없이 사용 금지 |
| `clippy::all` 통과 | 모든 clippy 경고 해결 |
| `rustfmt` 적용 | 자동 포매팅 |
| `pub` 최소화 | 필요한 것만 공개, 기본은 비공개 |

### 4.2 권장 패턴

```rust
// Good: Builder 패턴 for 복잡한 구성
let config = Config::builder()
    .tasks_path("./TASKS.md")
    .split_ratio(50)
    .build()?;

// Good: From/Into 트레이트 활용
impl From<RawTask> for Task {
    fn from(raw: RawTask) -> Self { ... }
}

// Good: Iterator 체이닝
let completed_count = phases.iter()
    .flat_map(|p| &p.tasks)
    .filter(|t| t.status == TaskStatus::Completed)
    .count();
```

### 4.3 비동기 규칙

```rust
// Tokio 비동기 패턴
// - select! 매크로로 다중 이벤트 소스 처리
// - 채널 (mpsc) 로 스레드 간 통신
// - 파일 감시 이벤트는 별도 태스크에서 수신

tokio::select! {
    event = keyboard_rx.recv() => handle_keyboard(event),
    event = file_rx.recv() => handle_file_change(event),
    _ = tick_interval.tick() => handle_tick(),
}
```

---

## 5. AI 소통 원칙

### 5.1 하나의 채팅 = 하나의 모듈

- 파서 구현 → 별도 대화
- UI 위젯 구현 → 별도 대화
- 컨텍스트가 길어지면 새 대화 시작

### 5.2 컨텍스트 명시

**좋은 예:**
> "TRD의 데이터 모델(섹션 4)을 기반으로 `src/data/state.rs`에 DashboardState 구조체를 구현해주세요. 04-database-design.md의 엔티티 정의를 참조하세요."

**나쁜 예:**
> "상태 모델 만들어줘"

### 5.3 프롬프트 템플릿

```
## 작업
{무엇을 해야 하는지}

## 참조 문서
- docs/planning/{문서명}.md 섹션 {번호}

## 제약 조건
- Rust 2021 edition
- clippy 통과 필수
- 테스트 포함

## 예상 결과
- {생성될 파일}
- {기대 동작}
```

---

## 6. 보안 체크리스트

### 6.1 필수 적용

- [ ] API 키는 환경변수로만 관리 (`ANTHROPIC_API_KEY`)
- [ ] `.env` 파일 `.gitignore`에 포함
- [ ] 사용자 입력(파일 경로) 검증: path traversal 방지
- [ ] 외부 파일 읽기 시 크기 제한 (DoS 방지)
- [ ] Anthropic API 호출 시 타임아웃 설정

### 6.2 Rust 특화

- [ ] `unsafe` 블록 0개 유지
- [ ] 의존성 보안 감사: `cargo audit`
- [ ] 퍼즈 테스트: TASKS.md 파서에 대해 `cargo-fuzz` 적용

---

## 7. 테스트 워크플로우

### 7.1 테스트 명령어

```bash
# 전체 테스트
cargo test

# 특정 모듈 테스트
cargo test data::tasks_parser

# 커버리지 측정
cargo tarpaulin --out Html

# 벤치마크
cargo bench

# 린트 + 포맷
cargo clippy -- -D warnings
cargo fmt -- --check

# 보안 감사
cargo audit
```

### 7.2 테스트 작성 규칙

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // 테스트 네이밍: 행위_조건_기대결과
    #[test]
    fn parse_tasks_md_with_valid_input_returns_phases() {
        let input = include_str!("../tests/fixtures/sample_tasks.md");
        let result = parse_tasks_md(input);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 3);
    }

    #[test]
    fn parse_tasks_md_with_empty_input_returns_empty_vec() {
        let result = parse_tasks_md("");
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
```

### 7.3 품질 게이트 (CI)

```yaml
# .github/workflows/ci.yml 체크 항목
- cargo fmt -- --check
- cargo clippy -- -D warnings
- cargo test
- cargo tarpaulin (커버리지 >= 80%)
- cargo audit (보안 취약점 0개)
- cargo build --release
```

---

## 8. Git 워크플로우

### 8.1 브랜치 전략

```
main              # 릴리즈 가능 상태
├── develop       # 개발 통합
│   ├── feat/parser       # TASKS.md 파서
│   ├── feat/gantt        # 간트차트 위젯
│   ├── feat/watcher      # 파일 감시
│   ├── feat/error-analysis # 에러 분석
│   └── fix/terminal-compat # 터미널 호환성
```

### 8.2 커밋 메시지

```
<type>(<scope>): <subject>

<body>
```

**타입:**
- `feat`: 새 기능
- `fix`: 버그 수정
- `refactor`: 리팩토링 (동작 변경 없음)
- `test`: 테스트 추가/수정
- `docs`: 문서
- `perf`: 성능 개선
- `chore`: 빌드/CI/의존성

**스코프 (모듈):**
- `parser`: TASKS.md/Hook 파서
- `ui`: TUI 렌더링
- `watcher`: 파일 감시
- `analysis`: 에러 분석
- `app`: 앱 코어

**예시:**
```
feat(parser): TASKS.md Phase/Task 기본 파싱 구현

- H2 헤딩에서 Phase 추출
- H3 헤딩에서 Task 추출
- [status] 태그 파싱
- @agent 이름 파싱
- tests/fixtures/sample_tasks.md 기반 테스트 추가
```

---

## 9. 의존성 관리

### 9.1 Cargo.toml 관리 규칙

```toml
[dependencies]
# 핵심 의존성 - 버전 고정
ratatui = "0.28"
crossterm = "0.28"
tokio = { version = "1", features = ["full"] }
clap = { version = "4", features = ["derive"] }

# 데이터 처리
serde = { version = "1", features = ["derive"] }
serde_json = "1"
notify = "6"
nom = "7"  # 또는 pest = "2"

# 에러 처리
anyhow = "1"
thiserror = "1"

# 로깅
tracing = "0.1"
tracing-subscriber = "0.3"

# 선택적 (AI 분석)
reqwest = { version = "0.12", features = ["json"], optional = true }

[features]
default = []
ai-analysis = ["dep:reqwest"]

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
proptest = "1"
insta = "1"
```

### 9.2 의존성 추가 규칙

| 규칙 | 설명 |
|------|------|
| 최소 의존성 | 꼭 필요한 crate만 추가 |
| 버전 고정 | major.minor 수준 고정 |
| feature 최소화 | 필요한 feature만 활성화 |
| 보안 감사 | 새 crate 추가 시 `cargo audit` 실행 |
| 라이선스 확인 | MIT/Apache-2.0 호환 확인 |

---

## Decision Log

| # | 결정 | 근거 |
|---|------|------|
| D1 | `.unwrap()` 금지 | 런타임 panic 방지, 안정적인 TUI 유지 |
| D2 | 레이어 분리 (UI/App/Data) | 모듈별 독립 테스트, 관심사 분리 |
| D3 | thiserror + anyhow 조합 | 라이브러리 에러(thiserror) + 앱 에러(anyhow) 분리 |
| D4 | Conventional Commits | 변경 이력 자동 추적, 릴리즈 노트 생성 |
| D5 | optional AI feature | API 키 없이도 기본 기능 동작 보장 |
