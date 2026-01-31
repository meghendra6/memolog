---
name: review
description: Review current changes, highlight risks, and suggest improvements.
argument-hint: "What should be reviewed? (scope/concerns)"
agent: Oracle
tools: ["changes", "problems", "usages", "codebase", "readFile"]
---

Task:
${input:task:Describe what you want reviewed.}

Use the `code-review` skill for the checklist and output format.
