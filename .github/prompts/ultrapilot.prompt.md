---
name: ultrapilot
description: Parallel autopilot for large, multi-part tasks
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
    "runSubagent",
  ]
---

<Critical_Principle>
**PARALLELIZE WITH OWNERSHIP**: Decompose into 2-5 independent tasks.
Assign file ownership per subagent to avoid conflicts.
</Critical_Principle>

# Ultrapilot Mode

Task:
${input:task:Describe the goal, constraints, and definition of done.}

## Protocol

| Phase | Action |
| ----- | ------ |
| Decompose | Split into 2-5 parallelizable subtasks |
| Assign | Map each subtask to a specific subagent + file ownership |
| Execute | Run subagents in parallel with clear boundaries |
| Integrate | Merge outputs, resolve conflicts, update TODOs |
| Validate | Run tests/checks and fix failures |

## Guardrails

| Rule | Reason |
| ---- | ------ |
| Use #tool:todos for task tracking | Shared status across subagents |
| Avoid overlapping files between subagents | Prevent merge conflicts |
| Delegate heavy edits to @Build | Orchestrator stays light |
| Use @Oracle for risk review | Catch edge cases early |

## Output Format (Final Message)

1. **Completed scope** (bullets)
2. **Files changed** (path list)
3. **How to run/test** (commands)
4. **Validation results** (what ran / what should run)
5. **Risks/follow-ups** (if any)
