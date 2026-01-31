---
name: pipeline
description: Sequential agent chaining with explicit handoff artifacts
argument-hint: "pipeline spec or goal (ex: explore -> plan -> build -> review)"
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
**SEQUENTIAL WITH ARTIFACTS**: Each stage must output a concise artifact for the next stage.
</Critical_Principle>

# Pipeline Mode

Task:
${input:task:Provide a pipeline spec or describe the goal.}

## Presets (if no pipeline provided)

- `review`: Explore → Oracle → CodeReviewer → Build (optional)
- `implement`: Planner → Build → TDDGuide
- `debug`: Explore → BuildFixer → Oracle
- `security`: Explore → SecurityReviewer → Build

## Protocol

| Step | Action |
| ---- | ------ |
| 1 | Parse pipeline or select a preset |
| 2 | Run each subagent in order with scoped instructions |
| 3 | Collect artifacts after each stage |
| 4 | Implement only in Build/BuildFixer stages |
| 5 | Validate and summarize |

## Output Format

- **Pipeline used**
- **Stage artifacts** (context, plan, risks, changes)
- **Final summary** (files, verification, follow-ups)
