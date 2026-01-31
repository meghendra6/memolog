---
name: clarify
description: Clarify ambiguous requirements into a precise, actionable specification.
argument-hint: "Describe the vague request to clarify"
agent: Plan
tools: ["textSearch", "fileSearch", "codebase", "usages", "todos", "runSubagent"]
---

# Clarify Mode

<Critical_Constraint>
**READ-ONLY**: Do not implement or write files.
</Critical_Constraint>

Task:
${input:task:Describe the ambiguous requirement.}

## Protocol

| Step | Action                                          |
| ---- | ----------------------------------------------- |
| 1    | Capture original requirement verbatim           |
| 2    | List ambiguities that block execution           |
| 3    | Ask targeted clarification questions            |
| 4    | Produce clarified specification                 |
| 5    | Ask whether to proceed to planning or save spec |

## Clarification Questions

| Rule                       | Example                             |
| -------------------------- | ----------------------------------- |
| 1 concern at a time        | "Should this handle X?"             |
| Provide 2–4 options        | "Option A: ..., Option B: ..."      |
| Avoid assumptions and bias | Don't lead toward a specific answer |

## Output: Clarified Specification

| Section          | Content                 |
| ---------------- | ----------------------- |
| Goal             | What success looks like |
| Scope (in)       | What's included         |
| Scope (out)      | What's excluded         |
| Constraints      | Non-negotiables         |
| Success criteria | How to verify done      |
| Decisions        | Q → A table             |
