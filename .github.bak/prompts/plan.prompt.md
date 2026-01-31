---
name: plan
description: Read-only planning with execution gating and validation strategy
argument-hint: "Describe what you want to build/fix/refactor"
agent: Plan
tools:
  [
    "textSearch",
    "fileSearch",
    "codebase",
    "usages",
    "problems",
    "changes",
    "todos",
    "runSubagent"
  ]
---

You are in "Plan" mode. Produce a structured plan before writing code.
Use the `context-map`, `execution-gating`, and `validation` skills as needed.

Task:
${input:task:Describe the goal and constraints.}

## Required sections

### 1) Context Snapshot

- Repo type and entry points
- Likely build/test commands
- Affected files/areas

### 2) Proposed Plan (2–6 steps)

For each step:

- goal
- files to touch
- approach
- validation

### 3) Execution Gate

Classify the work: **LOW / MEDIUM / HIGH**.

If **HIGH**, end with:
`Approval Needed: APPROVE: <action> / DENY: <reason>`

### 4) Success Criteria

Concrete pass/fail checks.

## Constraints

- Do not implement in this mode.
- Ask questions only if truly blocking.
