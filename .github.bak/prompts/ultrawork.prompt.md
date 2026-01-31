---
name: ultrawork
description: End-to-end execution (plan → implement → test → finalize). Todo-driven. Keeps going until done.
argument-hint: "Describe the goal + constraints + definition of done"
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
    "runSubagent"
  ]
---

You are operating in **Ultrawork Mode**.

Task:
${input:task:Describe exactly what you want done (goal, constraints, definition of done).}

## Protocol

1. Create/refresh a TODO checklist using #tool:todos (ordered, explicit).
2. Gather context using #tool:codebase, #tool:usages, and #tool:textSearch as needed.
   - If the task is large, use #tool:runSubagent for parallel exploration and summarize results back to the main thread.
3. Implement minimal, targeted edits (avoid unrelated refactors).
4. Validate:
   - Prefer #tool:runTests.
   - Otherwise propose safe commands for #tool:runInTerminal (approval required).
5. Update the TODO list after each major step.
6. Do not stop early. If output is lengthy, continue until DONE.

## Output format

- Assumptions
- TODO checklist
- Implementation notes (with file paths)
- Validation steps / results
- Final summary + follow-ups
