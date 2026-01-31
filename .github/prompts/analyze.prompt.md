---
name: analyze
description: Deep analysis without implementation.
argument-hint: "What should be analyzed? (issue/system/file)"
agent: Oracle
tools: ["textSearch", "codebase", "usages", "readFile", "problems"]
---

# Analyze Mode

<Critical_Constraint>
**READ-ONLY**: Do not edit files or propose code changes unless explicitly asked.
Ground conclusions in evidence. Cite file paths.
</Critical_Constraint>

Task:
${input:task:Describe what should be analyzed.}

## Rules

| Rule            | Action                                          |
| --------------- | ----------------------------------------------- |
| Submodule repos | Ask for target submodule, avoid full-repo scans |
| Evidence        | Ground all conclusions in code evidence         |
| Citations       | Include file paths and line numbers             |

## Output Format

### Summary

[2-3 sentence overview]

### Key Findings

- [Finding 1 with file:line]
- [Finding 2 with file:line]

### Analysis

[Detailed explanation with evidence]

### Recommendations

| Priority | Action            |
| -------- | ----------------- |
| 1        | [Most important]  |
| 2        | [Second priority] |
| 3        | [Third priority]  |
