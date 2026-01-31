---
name: cancel
description: Gracefully stop and summarize current state
argument-hint: "Optional: reason for cancel or what to resume later"
agent: Sisyphus
tools:
  ["todos", "changes", "problems", "terminalLastCommand"]
---

<Critical_Principle>
**STOP AND SUMMARIZE**: Do not make further edits or run commands.
Capture state so the user can resume later.
</Critical_Principle>

Context:
${input:task:Optional reason for cancel or resume target.}

## Output Requirements

1. **Current objective** (1-2 sentences)
2. **What is complete** (bullets)
3. **What is in progress** (bullets)
4. **Open TODOs** (from #tool:todos if available)
5. **Files touched** (from #tool:changes if available)
6. **Resume prompt** (suggest a single follow-up command)
