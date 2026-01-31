---
name: context-map
description: Build a minimal, high-signal map of the repository (entrypoints, architecture, build/test commands, constraints). Use when starting work on an unfamiliar codebase or when tasks fail due to missing context.
---

# Context Map

<Critical_Principle>
**MINIMIZE CONTEXT, MAXIMIZE SIGNAL**: Produce actionable overview without bloating context.
Focus on what enables correct implementation, skip what doesn't.
</Critical_Principle>

## When to Use

| Situation | Action |
|-----------|--------|
| First interaction with repo | MANDATORY |
| Before multi-step changes | Recommended |
| Build/test commands unclear | Required |
| Errors suggest missing knowledge | Required |

## Procedure

### Step 1: Identify Project Type

| File | Stack |
|------|-------|
| `package.json` | Node.js / JavaScript / TypeScript |
| `pyproject.toml`, `requirements.txt` | Python |
| `go.mod` | Go |
| `Cargo.toml` | Rust |
| `pom.xml`, `build.gradle` | Java |
| `Makefile` | Build automation |

### Step 2: Find Commands

Search in priority order:
1. `AGENTS.md` — validation commands section
2. `package.json` scripts — `npm run <script>` or `pnpm <script>`
3. `Makefile` — `make <target>`
4. `pyproject.toml` scripts — `poetry run <script>`
5. CI workflows — `.github/workflows/*.yml`

### Step 3: Locate Entry Points

| Type | Common Locations |
|------|------------------|
| Web apps | `src/main.*`, `src/index.*`, `app/`, `pages/` |
| APIs | `src/server.*`, `cmd/`, `main.go`, `app.py` |
| Libraries | `src/lib.*`, `src/index.*`, package entry |
| CLI tools | `bin/`, `cli.*`, `__main__.py` |

### Step 4: Identify Constraints

- Runtime versions
- Lint/format rules
- Directory conventions
- Test patterns

## Output Template

### Repo Summary
- **Type**: [language/framework]
- **Entry points**: [main files]
- **Key modules**: [important directories]

### Dev Commands

| Command | Script |
|---------|--------|
| Install | `...` |
| Lint | `...` |
| Typecheck | `...` |
| Test | `...` |
| Build | `...` |

### Constraints
- [Style/naming patterns]
- [Test structure]
- [Import conventions]

### Suggested Next Action
If planning: propose 2–6 step plan with validation per step.
