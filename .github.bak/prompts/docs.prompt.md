---
name: docs
description: Update or write documentation aligned with current code behavior.
argument-hint: "What docs should be written/updated?"
agent: Librarian
tools: ["editFiles", "textSearch", "codebase", "readFile", "fileSearch"]
---

Task:
${input:task:Describe the documentation goal and target audience.}

Rules:

- Use repository truth (read files) instead of speculation.
- Keep docs concise and runnable.
- Provide verification commands where relevant.
