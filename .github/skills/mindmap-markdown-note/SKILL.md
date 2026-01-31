---
name: mindmap-markdown-note
description: Produce Korean-first Markdown knowledge notes with a brainstormed, organized mindmap.
---

# Mindmap Markdown Note

<Critical_Principle>
**BRAINSTORM → ORGANIZE → EXPLAIN**: First diverge (generate diverse branches), then converge (normalize into organized tree), then explain with examples.
</Critical_Principle>

## When to Use

| Situation | Action |
|-----------|--------|
| Need mindmap-like outline with levels | Use this skill |
| Want brainstorm → organize → explain flow | Use this skill |
| Need scannable knowledge note | Use this skill |

## Output Contract (Markdown)

Start with:
- `# <Title>`
- A metadata block as a short list:
  - `- Audience: ...`
  - `- Goal: ...`
  - `- Assumptions: ...` (if any; otherwise "None")
- Then `## Mindmap` section with nested bullet lists.

Required sections (in this order):
1) `## Mindmap`
2) `## Explanation`
3) `## Examples`
4) `## Pitfalls & Checks`
5) `## Next Actions`

Mindmap rules:
- Use nested Markdown lists.
- Must include at least one branch that is 3+ levels deep.
- Each node line is short (recommend <= 90 chars); prefer adding sub-bullets instead of wrapping.
- Each node should be a noun phrase or a short statement; avoid vague labels.
- Include "Relationships" as part of the tree (e.g., "A vs B", "depends on", "trade-off").

Brainstorming rules:
- First generate diverse candidate branches (divergent).
- Then merge/normalize into an organized tree (convergent).
- If the user topic is narrow, still broaden via adjacent categories (history, alternatives, constraints, metrics).

Sentence style rules:
- Korean base text, but keep technical terms in English.
- Prefer short declarative sentences; avoid rhetorical or fluffy lines.
- Explain difficult concepts with "plain language + precise term" pattern:
  - Example: "쉽게 말해 ... 이고, 정확히는 <English term> ..."

Do/Don’t list:
- DO: use headings; use nested bullets; make assumptions explicit.
- DON’T: long paragraphs; tables by default; special glyph bullets; callout blocks.

## Example (synthetic)

# Dependency Injection 정리
- Audience: 신규 팀원
- Goal: DI 개념과 적용 포인트 이해
- Assumptions: None

## Mindmap
- 개념
  - 쉽게 말해 객체 연결 방식이고, 정확히는 dependency injection 원칙
  - 구성 요소
    - container
    - lifecycle
- 설계 선택지
  - constructor injection
  - setter injection
  - field injection
- 적용 지점
  - startup wiring
  - module boundaries
    - package layout
      - public API surface
- Relationships
  - A vs B: constructor injection vs service locator
  - depends on: interface + inversion of control
  - trade-off: testability vs setup cost

## Explanation
- 쉽게 말해 결합도를 낮추는 방식이고, 정확히는 dependency injection 패턴이다.
- 테스트에서는 mock을 쉽게 주입할 수 있다.

## Examples
- web server에서 controller가 service를 주입받는 경우
- CLI에서 parser가 config loader를 주입받는 경우

## Pitfalls & Checks
- [ ] 전역 container로 숨은 의존성이 생기지 않는가
- [ ] lifecycle 관리가 불명확하지 않은가
- [ ] circular dependency가 없는가
- [ ] 테스트에서 대체 구현 주입이 가능한가
- [ ] 설정 코드가 과도하게 복잡하지 않은가

## Next Actions
- 기존 모듈의 의존성 그래프를 그린다.
- constructor injection 우선 적용을 시도한다.
- 테스트 대체 구현을 준비한다.
