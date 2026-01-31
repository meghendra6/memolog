---
name: mindmapnote
description: "Write a Markdown mindmap-style knowledge note (brainstorm -> organize -> explain)."
argument-hint: "topic=... audience=... context=... constraints=... (optional)"
agent: Librarian
tools: ["textSearch", "readFile", "fileSearch", "codebase"]
---

# Mindmap Note

Use skill: #file:../skills/mindmap-markdown-note/SKILL.md

## Input

| Field       | Value                            |
| ----------- | -------------------------------- |
| Topic       | ${input:topic:Topic}             |
| Audience    | ${input:audience:Audience}       |
| Context     | ${input:context:Context}         |
| Constraints | ${input:constraints:Constraints} |

## Requirements

| Rule              | Requirement                                         |
| ----------------- | --------------------------------------------------- |
| Output Contract   | Follow exactly, keep required section order         |
| Flow              | Brainstorm → Organize → Explain                     |
| Language          | Korean base, English technical terms                |
| Mindmap           | At least 3 major branches (unless explicitly fewer) |
| Depth             | At least one branch with 3+ levels                  |
| Relationships     | Include as a branch in mindmap                      |
| Examples          | At least 2                                          |
| Pitfalls & Checks | At least 5 checklist items                          |
| Next Actions      | At least 3 actionable bullets                       |
| Missing input     | Infer defaults, list under Assumptions              |
| Questions         | Only if truly blocking                              |
| Style             | Short bullets and sentences, avoid long paragraphs  |
