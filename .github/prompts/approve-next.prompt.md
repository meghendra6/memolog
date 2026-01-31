---
name: approve-next
description: Approve and execute the next step incrementally with validation
argument-hint: "Specify which step to execute from the plan"
agent: Sisyphus
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

# Approve & Execute Next Step

Use after a plan is approved to execute one step with `execution-gating` + `validation`.

## Input

- Approved plan (or excerpt) + current step number

## Workflow

| Step | Action                                                             |
| ---- | ------------------------------------------------------------------ |
| 1    | Confirm approval state (if HIGH risk, require `APPROVE: <action>`) |
| 2    | Execute exactly **one** step                                       |
| 3    | Apply Validation Gate (run/provide commands, capture results)      |
| 4    | If failure, stop and fix                                           |
| 5    | Summarize and propose next step                                    |

## Output Template

### Step

<step N title>

### Changes

- [file]: [change description]

### Validation

| Command | Result             |
| ------- | ------------------ |
| `...`   | [output/exit code] |

### Next

- **Proposed next step**: [title]
- **Risk**: LOW / MEDIUM / HIGH
- **Action**: [proceed / require approval]
