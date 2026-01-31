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
    "todos"
  ]
---

# Approve & Execute Next Step

Use this prompt after a plan is produced and you want to execute it incrementally.
Use the `execution-gating` and `validation` skills as needed.

## Input contract

- You will be given:
  - the previously approved plan (or a plan excerpt)
  - the current step number to execute

## Required behavior (OpenAgents-style)

1. Confirm approval state:
   - If the plan (or current step) is **HIGH risk**, require: `APPROVE: <action>` before proceeding.
2. Execute exactly **one** step.
3. Apply the Validation Gate:
   - run or provide exact commands
   - capture results
4. Summarize:
   - What changed / files changed
   - Validation results
   - Risks or follow-ups
   - Next step proposal (do not execute it yet unless it is explicitly LOW risk and user asked to continue)

## Output template

### Step

<step N title>

### Changes

- ...

### Validation

- Commands:
  - ...
- Results:
  - ...

### Next

- Proposed next step: ...
