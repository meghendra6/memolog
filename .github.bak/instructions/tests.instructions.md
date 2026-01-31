---
name: Testing Standards
description: Guidance for test creation and modification.
applyTo: "**/*.test.*,**/*.spec.*,**/test/**,**/tests/**"
---

# Testing Standards

- Tests must be deterministic (no flakiness, no dependence on current time unless controlled).
- Prefer clear Arrange/Act/Assert structure.
- Test behavior, not implementation details.
- If you fix a bug, add a regression test.
- Keep test data minimal and readable.
