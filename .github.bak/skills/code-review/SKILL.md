---
name: code-review
description: Perform a structured review focusing on correctness, security, maintainability, and test coverage. Use this before finalizing a PR or when asked to review/refactor code.
---

# Code Review (Structured)

## Procedure
1) Scan the change summary or diff for scope and risk hotspots.
2) Review the focus areas below in order.
3) Produce findings and a clear approval stance.

## Focus areas
1) Correctness & edge cases
2) Security & secrets hygiene
3) Maintainability & readability
4) Performance (only where relevant)
5) Tests & validation adequacy

## Checklist
### Correctness
- Are invariants explicit?
- Are errors handled consistently?
- Are boundary cases covered?

### Security
- No secrets in code/logs
- Input validation/sanitization where needed
- Safe defaults for config and permissions

### Maintainability
- Naming is clear and consistent
- Complexity is appropriate
- Public APIs are stable (or changes are intentional and documented)

### Tests
- New behavior has tests
- Tests are deterministic and minimal
- Validation Gate passes (or commands are provided)

## Output template
### Review Summary
- Overall: approve / request changes

### Findings
- Must-fix:
  - ...
- Should-fix:
  - ...
- Nice-to-have:
  - ...

### Suggested Patch (if small)
- ...
