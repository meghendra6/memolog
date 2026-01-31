---
name: plan
description: Read-only planning with execution gating and validation strategy
argument-hint: "Describe what you want to build/fix/refactor"
agent: Plan
tools:
  ["textSearch", "fileSearch", "codebase", "usages", "problems", "changes", "todos", "runSubagent"]
---

# Plan Mode

<Critical_Constraint>
**READ-ONLY**: Do not implement in this mode.
Ask questions only if truly blocking.
</Critical_Constraint>

<Note>
This prompt is optional. In ultrawork, planning is performed internally.
</Note>

<Note>
If you need an interview-style planning flow, use `/planner`.
</Note>

Task:
${input:task:Describe the goal and constraints.}

Use skills: `context-map`, `execution-gating`, `validation`.

## Required Sections

### 1) Context Snapshot

| Item                 | Content              |
| -------------------- | -------------------- |
| Repo type            | [language/framework] |
| Entry points         | [main files]         |
| Build/test commands  | [commands]           |
| Affected files/areas | [list]               |

### 2) Constraints & Assumptions

| Type            | Content                          |
| --------------- | -------------------------------- |
| Non-negotiables | [scope limits, risk constraints] |
| Assumptions     | [explicit, mark unknowns]        |

### 3) Proposed Plan (2–6 steps)

For each step:

| Field      | Content                 |
| ---------- | ----------------------- |
| Goal       | What this step achieves |
| Files      | Which files to touch    |
| Approach   | How to implement        |
| Validation | How to verify           |

### 4) Execution Gate

Classify: **LOW / MEDIUM / HIGH**

If **HIGH**:

```
Approval Needed: APPROVE: <action> / DENY: <reason>
```

### 5) Success Criteria

Concrete pass/fail checks.
