---
name: docs
description: Update or write documentation aligned with current code behavior.
argument-hint: "What docs should be written/updated?"
agent: Librarian
tools: ["editFiles", "textSearch", "codebase", "readFile", "fileSearch"]
---

# Docs Mode

Task:
${input:task:Describe the documentation goal and target audience.}

## Workflow

| Step | Action                                                   |
| ---- | -------------------------------------------------------- |
| 1    | Use repository truth (read files) instead of speculation |
| 2    | Document the chosen approach                             |
| 3    | Keep docs concise and runnable                           |
| 4    | Provide verification commands                            |
| 5    | Cite sources when using external docs                    |

## Constraints

| Do                    | Don't                               |
| --------------------- | ----------------------------------- |
| Read files for truth  | Speculate                           |
| Document what exists  | Redesign or argue with requirements |
| Keep concise          | Over-document                       |
| Cite external sources | Skip attribution                    |
