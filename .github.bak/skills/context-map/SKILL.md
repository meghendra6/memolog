---
name: context-map
description: Build a minimal, high-signal map of the repository (entrypoints, architecture, build/test commands, constraints). Use this when starting work on an unfamiliar codebase or when tasks fail due to missing context.
---

# Context Map (MVI-first)

## Goal
Produce a minimal, actionable overview of the repo that improves correctness without bloating context.

## When to use
- First interaction with a repo
- Before planning a multi-step change
- When build/test commands are unclear
- When errors suggest missing architecture knowledge

## Procedure
1) Identify project type(s)
   - Inspect: `README`, `package.json`, `pyproject.toml`, `requirements*.txt`, `go.mod`, `Cargo.toml`, `pom.xml`, etc.

2) Determine build/test/format/typecheck commands
   - Look for: `scripts` section, Makefile targets, CI workflows, tox/nox configs.

3) Identify the main entrypoints and key modules
   - Examples:
     - Web: `src/main.*`, `app/`, `pages/`, routing, server start files
     - Python: `__main__.py`, `app.py`, `src/`, package root

4) Summarize constraints
   - Supported runtime versions
   - Lint rules / formatting
   - Directory conventions

## Output format
### Repo Summary
- Type: ...
- Entrypoints: ...
- Key modules: ...

### Dev Commands
- Install: ...
- Lint: ...
- Typecheck: ...
- Test: ...
- Build: ...

### Constraints
- ...

### Suggested Next Action
- If planning: propose a 2–6 step plan with validation per step.
