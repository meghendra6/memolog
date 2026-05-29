# Changelog

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
