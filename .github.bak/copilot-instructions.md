# Copilot Workspace Instructions (Sisyphus Harness)

## Non-negotiables

- Use GitHub Copilot + VS Code built-in capabilities only.
- Do NOT use MCP servers. Do NOT require paid external services.
- Prefer repository truth over guesswork.
- Never request, output, or log secrets.

## Plan-first operating model

- Analyze → Plan → Execute (one step) → Validate → Summarize → Confirm next step.
- Keep plans 2–6 steps with success criteria.
- Always validate before claiming DONE.

## Approval gating (HIGH risk only)

- Use the `execution-gating` skill to classify risk.
- If HIGH, stop and require: `APPROVE: <action>` or `DENY: <reason>`.
- Destructive or system-level actions always require approval.

## Prefer Agent Skills (on-demand detail)

- Use skills under `.github/skills/` for procedures:
  - `context-map`, `execution-gating`, `validation`, `code-review`.

## Core Tools

Prefer these tools for context gathering:
- `#codebase` - semantic workspace search
- `#textSearch` - exact string search
- `#usages` - safe symbol navigation (references/definitions)
- `#problems` - diagnostics and errors
- `#changes` - review diffs before concluding

## Ultrawork Mode (keyword-triggered)

- If the user includes `ultrawork`, keep a TODO list using `#todos` and iterate until complete.

## Code Quality

- Make minimal, surgical changes. Avoid unrelated refactors.
- Do not add verbose AI-style comments.
- Update tests when behavior changes.

## Output requirements

- Planning: include `Execution Gate` (LOW/MEDIUM/HIGH) + `Success criteria`.
- Implementation: include `What changed`, `Why`, `How validated`, `Next step`.
- Final summary: file paths + verification commands + follow-ups.
