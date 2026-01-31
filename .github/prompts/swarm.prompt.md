---
name: swarm
description: Parallel swarm execution with task claiming
argument-hint: "N:role <task> (ex: 4:builder 'fix all TS errors')"
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
**SWARM WITH CLAIMS**: Break into many small tasks and let subagents claim them.
No two subagents should edit the same files.
</Critical_Principle>

# Swarm Mode

Task:
${input:task:Describe the task and desired number of workers.}

## Protocol

| Phase | Action |
| ----- | ------ |
| Split | Create a TODO list of small, independent tasks |
| Claim | Assign each TODO to a subagent (A/B/C/...) |
| Execute | Run subagents in parallel within their file boundaries |
| Merge | Integrate outputs and resolve conflicts |
| Validate | Run tests/checks and fix failures |

## Guardrails

| Rule | Reason |
| ---- | ------ |
| Use #tool:todos to track claims | Prevent duplicate work |
| Assign explicit file ownership | Avoid conflicts |
| Delegate heavy edits to @Build | Orchestrator stays light |

## Output Format (Final Message)

1. **Completed scope** (bullets)
2. **Files changed** (path list)
3. **How to run/test** (commands)
4. **Validation results** (what ran / what should run)
5. **Risks/follow-ups** (if any)
