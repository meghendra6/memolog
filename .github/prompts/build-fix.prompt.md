---
name: build-fix
description: Fix build/compilation errors with minimal changes
agent: BuildFixer
tools: ["editFiles", "textSearch", "runInTerminal", "problems", "codebase"]
---

# Build Fix Prompt

Fix the current build/compilation errors as quickly as possible with minimal diffs.

## Workflow

| Step | Action                                |
| ---- | ------------------------------------- |
| 1    | Detect project type from config files |
| 2    | Run appropriate diagnostic commands   |
| 3    | Categorize errors by type             |
| 4    | Fix each error with minimal change    |
| 5    | Verify fix doesn't break other code   |

## Detect Project & Run Diagnostics

Identify the project type and run the corresponding check:

| Config File      | Stack      | Diagnostic Command                 |
| ---------------- | ---------- | ---------------------------------- |
| `package.json`   | Node.js/TS | Project scripts or `npm run build` |
| `pyproject.toml` | Python     | `ruff check .` or `mypy .`         |
| `Cargo.toml`     | Rust       | `cargo check` or `cargo build`     |
| `go.mod`         | Go         | `go build ./...` or `go vet ./...` |
| `Makefile`       | Any        | `make` or `make check`             |

## Fix Strategy

### DO

- Add missing type annotations/hints
- Add null/error checks where needed
- Fix imports/exports/modules
- Update type definitions
- Minimal, surgical changes

### DON'T

- Refactor unrelated code
- Change architecture
- Add new features
- Rename without necessity

## Output

After fixing:

- Initial error count
- Errors fixed
- Remaining issues (if any)
- Verification commands run
