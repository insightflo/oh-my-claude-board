# User Flow (사용자 흐름도)

> oh-my-claude-board 대시보드의 핵심 사용자 여정

---

## MVP 캡슐

| # | 항목 | 내용 |
|---|------|------|
| 1 | 목표 | Claude Code 오케스트레이션 진행 상황을 터미널에서 실시간 시각화 |
| 2 | 페르소나 | Claude Code 헤비 유저 |
| 3 | 핵심 기능 | FEAT-1: Watch 모드, FEAT-2: 간트차트, FEAT-3: 에러 AI 분석 |
| 4 | 성공 지표 (노스스타) | GitHub Stars 100+ |
| 5 | 입력 지표 | 주간 다운로드 수, 이슈/PR 참여 수 |
| 6 | 비기능 요구 | 렌더링 60fps, 파일 변경 감지 < 500ms |
| 7 | Out-of-scope | Wrapper 모드, 비용 트래커, Slack 연동 |
| 8 | Top 리스크 | Rust 개발 속도 |
| 9 | 완화/실험 | 간트차트 프로토타입 먼저 구현 |
| 10 | 다음 단계 | Ratatui 프로토타입 구현 |

---

## 1. 전체 사용자 여정 (Overview)

```mermaid
graph TD
    A[터미널 열기] --> B{oh-my-claude-board 설치됨?}
    B -->|No| C[FEAT-0: cargo install]
    B -->|Yes| D[oh-my-claude-board watch 실행]
    C --> D
    D --> E{TASKS.md 존재?}
    E -->|No| F[에러: TASKS.md not found]
    E -->|Yes| G[대시보드 + Claude Code 분할 표시]
    F --> D
    G --> H[FEAT-2: 간트차트 모니터링]
    H --> I{이벤트 감지?}
    I -->|Task 상태 변경| J[간트차트 업데이트]
    I -->|에러 발생| K[FEAT-3: 에러 감지 + AI 분석]
    I -->|완료| L[전체 완료 표시]
    J --> H
    K --> M{재시도?}
    M -->|Yes| N[태스크 재시도]
    M -->|No| H
    N --> H
    L --> O[q로 종료]
```

---

## 2. FEAT-0: 설치 + 첫 실행 플로우

```mermaid
graph TD
    A[cargo install oh-my-claude-board] --> B[설치 완료]
    B --> C[프로젝트 디렉토리로 이동]
    C --> D[oh-my-claude-board watch]
    D --> E{TASKS.md 감지?}

    E -->|자동 감지| F[파일 감시 시작]
    E -->|못 찾음| G[--tasks 옵션으로 경로 지정 안내]
    G --> H[oh-my-claude-board watch --tasks ./path/TASKS.md]
    H --> F

    F --> I{Hook 이벤트 디렉토리?}
    I -->|자동 감지| J[Hook 감시 시작]
    I -->|못 찾음| K[TASKS.md만으로 동작 - 경고 표시]

    J --> L[대시보드 렌더링 시작]
    K --> L
```

---

## 3. FEAT-1: Watch 모드 모니터링 플로우

```mermaid
graph TD
    A[Watch 모드 시작] --> B[화면 분할]
    B --> C[좌: 대시보드 / 우: Claude Code 출력]

    C --> D[TASKS.md 초기 파싱]
    D --> E[Phase/Task 트리 구성]
    E --> F[간트차트 초기 렌더링]

    F --> G{파일 변경 이벤트?}

    G -->|TASKS.md 변경| H[재파싱]
    H --> I[상태 모델 업데이트]
    I --> J[간트차트 리렌더]
    J --> G

    G -->|Hook 이벤트| K[JSON 파싱]
    K --> L[에이전트 상태 업데이트]
    L --> J

    G -->|키보드 입력| M[UI 인터랙션 처리]
    M --> J

    G -->|타이머 tick| N[경과 시간 업데이트]
    N --> J
```

---

## 4. FEAT-2: 간트차트 인터랙션 플로우

```mermaid
graph TD
    A[간트차트 표시 중] --> B{키보드 입력?}

    B -->|j/↓| C[다음 태스크로 커서 이동]
    B -->|k/↑| D[이전 태스크로 커서 이동]
    B -->|Enter| E[태스크 상세 패널 열기]
    B -->|Space| F{Phase 행?}

    F -->|Yes| G[Phase 접기/펼치기 토글]
    F -->|No| E

    E --> H[하단 패널에 상세 정보 표시]
    H --> I[에이전트명, 시작 시간, 상태, 로그]

    C --> A
    D --> A
    G --> A

    I --> J{에러가 있는 태스크?}
    J -->|Yes| K[AI 분석 결과 표시 + 재시도 버튼]
    J -->|No| A

    K --> L{r 키 입력?}
    L -->|Yes| M[태스크 재시도]
    L -->|No| A
    M --> A
```

---

## 5. FEAT-3: 에러 감지 + AI 분석 플로우

```mermaid
graph TD
    A[에러 이벤트 수신] --> B[에러 태스크 빨간색 표시]
    B --> C[자동으로 에러 태스크 포커스]
    C --> D[규칙 기반 패턴 매칭]

    D --> E{패턴 매칭 성공?}

    E -->|Yes| F[분석 결과 표시]
    E -->|No| G{API 키 설정됨?}

    G -->|Yes| H[Anthropic API 호출]
    G -->|No| I[일반 에러 메시지 표시]

    H --> J{API 응답 성공?}
    J -->|Yes| F
    J -->|No| I

    F --> K[하단 패널에 분석 결과 + 재시도 버튼]
    I --> K

    K --> L{r 키 입력?}
    L -->|Yes| M[재시도 명령 생성]
    M --> N[TASKS.md에 재시도 상태 기록]
    N --> O[간트차트 업데이트]
    L -->|No| P[다른 태스크로 이동]
```

---

## 6. 화면 목록 (Screen Inventory)

| 화면 ID | 화면명 | FEAT | 진입점 | 주요 액션 |
|---------|--------|------|--------|----------|
| S-01 | 대시보드 (메인) | ALL | oh-my-claude-board watch | 간트차트 + 상세 + Claude 출력 |
| S-02 | 간트차트 패널 | FEAT-2 | S-01 좌측 상단 | Phase/Task 트리 탐색 |
| S-03 | 태스크 상세 패널 | FEAT-2 | S-02에서 Enter | 에이전트/로그/에러 확인 |
| S-04 | 에러 분석 뷰 | FEAT-3 | S-03 (에러 태스크) | AI 분석 확인 + 재시도 |
| S-05 | Claude Code 출력 | FEAT-1 | S-01 우측 | 실행 로그 스크롤 |
| S-06 | 도움말 오버레이 | - | ? 키 입력 | 키바인딩 확인 |
| S-07 | 에러 화면 | - | TASKS.md 미발견 | 경로 지정 안내 |

---

## 7. 리텐션 루프

```mermaid
graph TD
    A[오케스트레이션 실행] --> B[oh-my-claude-board watch]
    B --> C[실시간 모니터링]
    C --> D[에러 발생 시 즉시 대응]
    D --> E[작업 완료]
    E --> F[결과 확인 - 전체 완료율]

    F --> G{다음 오케스트레이션?}
    G -->|Yes| A
    G -->|No| H[종료]

    I[습관 형성] --> J[오케스트레이션 실행 = oh-my-claude-board watch 자동 연상]
    J --> A
```

> TUI 대시보드의 리텐션은 "오케스트레이션을 실행할 때마다 자연스럽게 함께 사용"하는 습관 형성에 있다. 별도의 푸시 알림이나 리마인더가 필요 없으며, 도구 자체의 유용성이 리텐션을 만든다.

---

## 8. 에러 처리 플로우

```mermaid
graph TD
    A[에러 발생] --> B{에러 유형?}

    B -->|TASKS.md 파싱 실패| C[마지막 유효 상태 유지 + 경고 표시]
    B -->|Hook 디렉토리 없음| D[TASKS.md 전용 모드로 전환]
    B -->|파일 감시 끊김| E[자동 재연결 시도]
    B -->|터미널 크기 부족| F[최소 크기 경고 + 축소 레이아웃]
    B -->|API 호출 실패| G[규칙 기반 분석으로 폴백]

    C --> H[상태바에 경고 아이콘 표시]
    D --> H
    E --> I{재연결 성공?}
    I -->|Yes| J[정상 모드 복구]
    I -->|No - 3회 실패| K[수동 재시작 안내]
    F --> H
    G --> H
```

---

## Decision Log

| # | 결정 | 근거 |
|---|------|------|
| D1 | 설치 후 Zero-config 작동 | 진입 장벽 최소화, TASKS.md 자동 감지 |
| D2 | 에러 시 자동 포커스 | 에러 인지 시간 최소화 |
| D3 | 파싱 실패 시 마지막 유효 상태 유지 | 일시적 파일 변경으로 인한 깜빡임 방지 |
| D4 | vim 스타일 키바인딩 (j/k) | 타겟 사용자(개발자)의 터미널 사용 습관 |
