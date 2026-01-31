---
name: fix
description: Fix errors using Problems/Test Failure context and validate.
argument-hint: "What is broken? (error message / failing test / expected behavior)"
agent: Sisyphus
tools:
  [
    "editFiles",
    "textSearch",
    "runInTerminal",
    "runTests",
    "problems",
    "testFailure",
    "codebase",
    "usages",
    "changes",
    "todos"
  ]
---

Task:
${input:task:Describe the error/failure and expected behavior.}

Protocol:

1. Pull diagnostics using #tool:problems and/or #tool:testFailure.
2. Locate root cause using #tool:usages + #tool:codebase.
3. Implement minimal fix.
4. Validate with #tool:runTests (preferred).
5. Add regression test if appropriate.
6. Update TODO checklist and finalize with verification steps.
