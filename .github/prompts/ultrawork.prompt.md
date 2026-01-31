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
    "runSubagent",
  ]
---

<Critical_Principle>
**ONE REQUEST, COMPLETE RESULT**: Copilot charges per request.
Complete end-to-end unless blocked. Do NOT ask for `/plan` or `/review`.
</Critical_Principle>

# Ultrawork Mode

Task:
${input:task:Describe exactly what you want done (goal, constraints, definition of done).}

## Protocol

| Phase         | Action                                                    |
| ------------- | --------------------------------------------------------- |
| Context/Recon | Gather ALL relevant context                               |
| Plan          | Create TODO list via #tool:todos                          |
| Review        | Internal critic pass (delegate to @Oracle or self-review) |
| Implement     | Make minimal, surgical changes                            |
| Validate      | Run tests, fix failures before proceeding                 |
| Iterate       | Repeat until all TODOs complete                           |
| Finalize      | Summary with verification commands                        |

## Tooling Rules

| Rule                                       | Reason                |
| ------------------------------------------ | --------------------- |
| Create/update/close TODOs via #tool:todos  | Track progress        |
| Prefer #tool:usages and LSP navigation     | Safe refactors        |
| Run validation with #tool:runTests         | Verify changes        |
| Delegate multi-file edits to @Build        | Orchestrator stays light |
| Scope to submodule if repo uses submodules | Avoid expensive scans |

## Delegation

| Agent               | Purpose                     |
| ------------------- | --------------------------- |
| @Explore/@Librarian | Recon and context gathering |
| @Oracle             | Architecture critique       |
| @Build              | Heavy implementation        |

## Output Format (Final Message)

1. **Completed scope** (bullets)
2. **Files changed** (path list)
3. **How to run/test** (commands)
4. **Validation results** (what ran / what should run)
5. **Risks/follow-ups** (if any)

## Skill References

- #file:../skills/execution-gating/SKILL.md
- #file:../skills/validation/SKILL.md
- #file:../skills/code-review/SKILL.md
- #file:../skills/context-map/SKILL.md
