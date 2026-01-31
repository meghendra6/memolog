---
name: validation
description: Run the tightest relevant validation gate (lint, typecheck, tests, build, smoke) for the current repo. Use after any code change or when diagnosing failures.
---

# Validation Gate

<Critical_Principle>
**NO COMPLETION WITHOUT VERIFICATION**: Never claim "done" without running validation.
"Looks good" is not evidence. Run the command, read the output, THEN claim the result.
</Critical_Principle>

## When to Use

| Situation | Action |
|-----------|--------|
| After ANY implementation step | MANDATORY |
| Debugging failures | Required |
| Before claiming completion | MANDATORY |

## Procedure

### Step 1: Detect Ecosystem

| Stack | Config Files |
|-------|-------------|
| JS/TS | `package.json`, `tsconfig.json`, `eslint.config.*` |
| Python | `pyproject.toml`, `pytest.ini`, `ruff.toml`, `mypy.ini` |
| Go | `go.mod`, `Makefile` |
| Rust | `Cargo.toml` |

### Step 2: Choose Minimal Sufficient Gate

Prefer project's standard commands (scripts/Makefile/CI).

### Step 3: Run in Order

| Priority | Gate | Purpose |
|----------|------|---------|
| 1 | Format/Lint | Style and static analysis |
| 2 | Typecheck | Type safety |
| 3 | Unit tests | Behavior correctness |
| 4 | Build | Compilation success |
| 5 | Smoke test | Runtime sanity (if applicable) |

### Step 4: Handle Failures

If command unavailable: explain fallback (static reasoning + targeted tests).

## Verification Iron Law

| Claim | Requires | NOT Sufficient |
|-------|----------|----------------|
| Tests pass | Test output: 0 failures | "Should pass" |
| Linter clean | Linter output: 0 errors | Partial check |
| Build succeeds | Build exit 0 | Logs look good |

## Output Template

### Validation
- **Selected gate**: [lint/test/build]
- **Commands**:
  - `...`
- **Results**:
  - [actual output/exit codes]
- **If failures**:
  - Root cause: [hypothesis]
  - Fix plan: [steps]
