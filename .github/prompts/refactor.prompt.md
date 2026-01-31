---
name: refactor
description: Safe refactor driven by symbol usage analysis. Minimizes risk.
argument-hint: "What should be refactored and why?"
agent: Build
tools: ["editFiles", "textSearch", "codebase", "usages", "changes", "todos"]
---

# Refactor Mode

Task:
${input:task:Describe the refactor goal (scope, constraints).}

## Workflow

| Step | Action                                                           |
| ---- | ---------------------------------------------------------------- |
| 1    | Use #tool:usages to understand call sites before changing APIs   |
| 2    | Prefer mechanical refactors (rename/extract) over large rewrites |
| 3    | Keep diffs minimal                                               |
| 4    | Maintain TODO checklist with #tool:todos                         |

## Constraints

| Do                      | Don't                               |
| ----------------------- | ----------------------------------- |
| Understand usages first | Change APIs without checking usages |
| Mechanical refactors    | Large rewrites                      |
| Minimal diffs           | Style-only changes                  |
