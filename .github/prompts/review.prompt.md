---
name: review
description: Review current changes, highlight risks, and suggest improvements.
argument-hint: "What should be reviewed? (scope/concerns)"
agent: CodeReviewer
tools: ["changes", "problems", "usages", "codebase", "readFile"]
---

# Code Review

Task:
${input:task:Describe what you want reviewed.}

<Note>
This prompt is optional. In ultrawork, review is performed internally.
</Note>

## Rules

| Rule            | Action                                               |
| --------------- | ---------------------------------------------------- |
| Submodule repos | Ask for target submodule, avoid full-repo `#changes` |
| Findings        | Cite file paths with line numbers                    |

## Checklist

Use the `code-review` skill for the full checklist and output format.

## Output Format

### Summary

- **Verdict**: approve / request changes

### Findings

**Must-fix:**

- [file:line] Issue

**Should-fix:**

- [file:line] Issue

**Nice-to-have:**

- [file:line] Issue
