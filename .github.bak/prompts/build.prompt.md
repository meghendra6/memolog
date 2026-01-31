---
name: build
description: Implementation mode: apply a plan, edit code, run tests, and finalize.
argument-hint: "Paste the plan or describe the intended change"
agent: Build
tools: ["editFiles", "textSearch", "runInTerminal", "runTests", "codebase", "usages", "problems", "changes", "todos"]
---

Task:
${input:task:Paste the plan or describe the intended change.}

Rules:

- Execute exactly one step at a time.
- Use the `execution-gating` skill for approval decisions.
- Use the `validation` skill after each step.
- If the Execution Gate is HIGH, require `APPROVE: <action>` before proceeding.
- Keep diffs minimal and reviewable.
