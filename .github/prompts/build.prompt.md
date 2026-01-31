---
name: build
description: "Implementation mode: apply a plan, edit code, run tests, and finalize."
argument-hint: "Paste the plan or describe the intended change"
agent: Build
tools:
  [
    "editFiles",
    "textSearch",
    "runInTerminal",
    "runTests",
    "codebase",
    "usages",
    "problems",
    "changes",
    "todos",
  ]
---

# Build Mode

Task:
${input:task:Paste the plan or describe the intended change.}

## Workflow

| Step | Action                                              |
| ---- | --------------------------------------------------- |
| 1    | Execute exactly one step at a time                  |
| 2    | Use `execution-gating` skill for approval decisions |
| 3    | Use `validation` skill after each step              |
| 4    | If validation fails, stop and fix                   |
| 5    | If HIGH risk, require `APPROVE: <action>`           |

## Constraints

| Do                                | Don't                            |
| --------------------------------- | -------------------------------- |
| Keep diffs minimal and reviewable | Skip validation                  |
| Validate each step                | Proceed on HIGH without approval |
