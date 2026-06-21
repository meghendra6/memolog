# Changelog

## [Unreleased]

### 한국어
- 일정 메타데이터만 있고 본문이 없는 노트 라인(예: 단독 `@sched(...)`/`@time(...)`)이 조용히 사라지지 않고 `(scheduled)`/`(reminder)` 플레이스홀더로 아젠다에 표시됨
- 잘못된 테마 색상 값을 조용히 무시하지 않고 시작 시 알림으로 표시 (`config.toml` 확인 안내)
- 포모도로 완료 알림을 아무 키나 눌러 닫을 수 있도록 개선 (기존: Esc만)
- 아젠다/태스크 빈 상태 메시지가 현재 필터(및 아젠다 날짜)를 명시
- 접힌 항목 표시(▶)를 강조 색상으로 표시해 스캔 시 구분 용이
- 짧은 터미널에서 고정(pinned) 섹션이 타임라인을 과도하게 밀어내지 않도록 높이 제한
- Mood/Carryover 팝업이 테마를 따르도록 수정, 팝업 푸터 단축키 표기 일관화
- 일일 노트 템플릿 플레이스홀더 추가: `{{date_short}}`, `{{month}}`, `{{year}}`, `{{week}}`, `{{iso_week}}`, `{{doy}}`
- 캡처 시 자동 추가된 토큰을 토스트로 표시 (예: `added @sched(...) @time(...)`)
- 컴포저 Visual 모드 `o`로 선택 영역의 반대쪽 끝으로 커서 이동 (앵커 교체)
- `[editor] link_complete_max_items` 설정 추가 (위키링크 자동완성 표시 개수, 기본 12)
- 메모 뷰어가 GFM 표를 정렬·폭 보정된 박스 드로잉 그리드로 렌더링
- 메모 뷰어 Reading Mode 추가: `z`로 전체화면·중앙 정렬·크롬 숨김 집중 읽기 (Esc 닫기)
- Navigate 모드에서 포커스되지 않은 패널 내용을 흐리게(dim) 처리해 집중도 향상 (focus_mode에서는 제외)
- 마크다운 뷰어 가독성 개선: 제목 레벨별 색/강조 차등화, 코드 블록 언어 배지 강조, 인라인 코드 배경 칩, 중첩 인용문 들여쓰기
- 확장 체크박스 상태 렌더링: `[/]` 진행중(◐), `[-]` 취소(~, 취소선), `[>]` 연기(→), `[!]` 중요(! 강조) — Obsidian 호환
- Chrome 토글(`\`): 패널 테두리/타이틀과 상태바를 숨겨 최대 집중 작업 공간 확보
- 컴팩트 상태바 옵션(`[ui] minimal_status_bar`): 모드/포커스/스트릭/진행도만 표시
- 포모도로 집중 세션(`[pomodoro] auto_focus_session`): 타이머 시작 시 크롬을 자동으로 숨기고 종료 시 복원

### English
- Note lines that carry only schedule metadata (e.g. a bare `@sched(...)`/`@time(...)`) now appear in the agenda with a `(scheduled)`/`(reminder)` placeholder instead of being silently dropped
- Invalid theme color values are surfaced in a startup notice (check `config.toml`) instead of silently falling back to the terminal default
- The pomodoro completion alert can be dismissed with any key (previously Esc only)
- Agenda/Tasks empty-state copy now names the active filter (and the agenda date)
- The collapsed fold marker (▶) is accented so folded entries stand out while scanning
- The pinned section height is capped so it can no longer squash the timeline on short terminals
- Mood/Carryover popups now follow the active theme; popup footer keybinding notation standardized
- Daily-note template gains `{{date_short}}`, `{{month}}`, `{{year}}`, `{{week}}`, `{{iso_week}}`, `{{doy}}` placeholders
- The capture toast now names the tokens it inferred (e.g. `added @sched(...) @time(...)`)
- Vim Visual mode `o` swaps the cursor to the opposite end of the selection
- New `[editor] link_complete_max_items` setting (wikilink autocomplete cap, default 12)
- The memo viewer now renders GFM tables as alignment-aware, width-clamped box-drawing grids
- New Reading Mode in the memo viewer: `z` toggles a fullscreen, centered, chrome-free focus view (Esc to close)
- In Navigate mode, unfocused panel content is dimmed so attention settles on the active panel (skipped in focus_mode)
- Markdown viewer readability: per-level heading styling, an emphasized code-block language badge, inline-code background "chips", and indented nested blockquotes
- Extended checkbox states render distinct glyphs: `[/]` in-progress (◐), `[-]` cancelled (~, struck through), `[>]` deferred (→), `[!]` important (!) — Obsidian-compatible
- Chrome toggle (`\`): hide panel borders/titles and the status bar for a maximally distraction-free workspace
- Compact status bar option (`[ui] minimal_status_bar`): show only mode/focus/streak/progress
- Pomodoro focus session (`[pomodoro] auto_focus_session`): starting a timer auto-hides chrome and restores it when the timer ends

## [1.4.0]

### 한국어
- 세션 콘텐츠 캐시 및 리더 캐시 라우팅 추가로 로드 성능 개선
- `ActivePopup` 열거형 리팩터링으로 팝업 상태 관리 단순화
- Obsidian 스타일 `[[wikilinks]]` 지원: 백링크, 인라인 하이라이트(`[theme] link` 토큰), `[[alias|target]]` 문법
- `Shift+L` 링크/백링크 팝업 및 메모 뷰어 내 필터링 지원
- `Shift+M` 위키링크 에고 그래프 뷰 (노드 탐색, 이전 기록, 백링크 열기)
- `[[` 컴포저 자동완성: 기존 링크 대상 제안 및 Enter/Tab으로 삽입
- `Shift+R` 리뷰 & 인사이트: 일/주/월 요약, 포모도로 시간 추적, Markdown/CSV 내보내기 (`<log_path>/exports/`)
- 캡처 인텔리전스: 자연어 날짜/시간 파싱 (`[capture] nl_parse`, 기본값 활성화) 및 일일 노트 템플릿 (`[capture] daily_template`)

### English
- Session content cache and readers cache routing for improved load performance
- `ActivePopup` enum refactor simplifying popup state management
- Obsidian-style `[[wikilinks]]` with backlinks, inline highlight (`[theme] link` token), and `[[target|alias]]` syntax
- `Shift+L` Links popup with topic→backlinks and date→jump; filtered view from the memo viewer
- `Shift+M` wikilink ego-graph view: navigate neighbors, go back through history, open backlinks or jump to date
- `[[` composer autocomplete: suggests existing link targets; Enter/Tab to insert, Esc to dismiss
- `Shift+R` Review & insight: Day/Week/Month summary, pomodoro time tracking, Markdown and CSV export to `<log_path>/exports/`
- Capture intelligence: natural-language date/time parsing (`[capture] nl_parse`, on by default) and daily-note templates (`[capture] daily_template`)

## [1.3.7]

### 한국어
- 패키지 버전 메타데이터를 `1.3.7` 릴리스로 정렬
- Capture Inbox 워크플로 추가: `Ctrl+N` Quick Capture가 `#inbox` 태그로 저장되고, Command Palette의 `Open inbox`로 빠르게 검토 가능
- 메모 뷰어, 마크다운 표시, Visual 편집, 이미지/Zen 편집 흐름의 안정성 개선
- 키바인딩 충돌 정규화와 workflow shell 기반 개선 포함

### English
- Aligned package version metadata for the `1.3.7` release
- Added Capture Inbox workflow: `Ctrl+N` Quick Capture saves with `#inbox`, and `Open inbox` in the Command Palette opens captured notes for review
- Improved memo viewer, markdown display, Visual editing, and image/Zen editing reliability
- Includes keybinding conflict normalization and workflow shell foundation updates

## [1.1.0]

### 한국어
- 테마 토큰화 및 프리셋 5종 추가, `T` 키로 Theme Switcher 팝업에서 즉시 미리보기/적용 가능 (`[ui] theme_preset`)
- Composer UX 개선: 중앙 정렬 레이아웃 + 상태바 HUD, 개선된 placeholder, 실시간 불릿 리스트 렌더링
- Composer 가독성 강화: 커서 라인 강조 및 라인 번호 가터(옵션, `ui.line_numbers`)
- Navigate/검색/태그/도움말 UI 현대화
- 타임라인 엔트리 삭제 팝업 추가 (`x` 키, 확인 후 삭제)
- 내부 정리: 경고 제거 등 유지보수성 개선

### English
- Theme tokenization with 5 presets; Theme Switcher popup via `T` with live preview and config support (`[ui] theme_preset`)
- Composer UX upgrades: centered layout + status bar HUD, improved placeholder, real-time bullet list rendering
- Composer clarity: cursor-line highlight and optional line-number gutter (`ui.line_numbers`)
- Navigate/Search/Tags/Help UI modernization
- Timeline entry delete confirmation popup (`x` key)
- Maintenance: warning cleanup and internal polish
