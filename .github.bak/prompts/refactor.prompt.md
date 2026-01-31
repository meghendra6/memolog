---
name: refactor
description: Safe refactor driven by symbol usage analysis. Minimizes risk.
argument-hint: "What should be refactored and why?"
agent: Build
tools: ["editFiles", "textSearch", "codebase", "usages", "changes", "todos"]
---

Task:
${input:task:Describe the refactor goal (scope, constraints).}

Rules:

- Use #tool:usages to understand call sites before changing APIs.
- Prefer mechanical refactors (rename/extract) over large rewrites.
- Keep diffs minimal; avoid style-only changes.
- Maintain a TODO checklist with #tool:todos.
