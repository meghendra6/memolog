---
name: validation
description: Run the tightest relevant validation gate (lint, typecheck, tests, build, smoke) for the current repo. Use this after any code change or when diagnosing failures.
---

# Validation Gate

## Goal
Prevent "looks good" changes that fail CI or break runtime behavior.

## When to use
- After any implementation step
- When debugging failures
- Before claiming completion

## Procedure
1) Detect the ecosystem (prefer repo configs over guesswork)
   - JS/TS: `package.json` scripts, `pnpm-lock.yaml`, `yarn.lock`, `tsconfig.json`, eslint/prettier configs
   - Python: `pyproject.toml`, `pytest.ini`, `tox.ini`, `ruff.toml`, `mypy.ini`
   - Go: `go.mod`, `Makefile`, CI
   - Rust: `Cargo.toml`

2) Choose the minimal sufficient gate
   - Always prefer the project's standard commands first (scripts/Makefile/CI).

3) Run (or provide) commands in this order (adjust to repo):
   - Format/Lint
   - Typecheck
   - Unit tests
   - Build
   - Targeted smoke test (if applicable)

4) If a command is unavailable, explain the fallback (static reasoning + targeted tests).

## Output template
### Validation
- Selected gate: ...
- Commands:
  - ...
- Results:
  - ...
- If failures:
  - Root cause hypothesis: ...
  - Fix plan: ...
