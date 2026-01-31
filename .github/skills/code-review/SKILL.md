---
name: code-review
description: Perform a structured review focusing on correctness, security, maintainability, and test coverage. Use before finalizing a PR or when asked to review/refactor code.
---

# Code Review

<Critical_Principle>
**EVIDENCE-BASED REVIEW**: Cite specific file paths and line numbers for every finding.
No vague claims. Every issue must be traceable to code.
</Critical_Principle>

## Procedure

| Step | Action |
|------|--------|
| 1 | Scan diff for scope and risk hotspots |
| 2 | Review focus areas in priority order |
| 3 | Produce findings with clear approval stance |

## Focus Areas (Priority Order)

| Priority | Area | Key Questions |
|----------|------|---------------|
| 1 | Correctness | Invariants explicit? Errors handled? Edge cases covered? |
| 2 | Security | No secrets in code/logs? Input validated? Safe defaults? |
| 3 | Maintainability | Clear naming? Appropriate complexity? Stable APIs? |
| 4 | Performance | Only where relevant to the change |
| 5 | Tests | New behavior tested? Deterministic? Coverage adequate? |

## Review Checklist

### Correctness
- [ ] Invariants are explicit
- [ ] Error handling is consistent
- [ ] Boundary cases are covered

### Security
- [ ] No secrets in code or logs
- [ ] Input validation/sanitization present
- [ ] Safe defaults for config and permissions

### Maintainability
- [ ] Naming is clear and consistent
- [ ] Complexity is appropriate
- [ ] Public APIs are stable or changes documented

### Tests
- [ ] New behavior has tests
- [ ] Tests are deterministic
- [ ] Validation Gate passes (or commands provided)

## Verdicts

| Verdict | Meaning |
|---------|---------|
| **APPROVE** | No blocking issues, ready to merge |
| **REQUEST CHANGES** | Must-fix issues identified |

## Output Template

### Review Summary
- **Verdict**: approve / request changes
- **Risk Level**: LOW / MEDIUM / HIGH

### Findings

**Must-fix:**
- [file:line] Issue description

**Should-fix:**
- [file:line] Issue description

**Nice-to-have:**
- [file:line] Issue description

### Suggested Patch (if small)
```diff
...
```
