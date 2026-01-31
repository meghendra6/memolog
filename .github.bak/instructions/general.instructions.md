---
name: General Engineering Guidelines
description: Cross-language rules used when Copilot creates or modifies files.
applyTo: "**/*"
---

# General Engineering Guidelines

Apply the repository-wide rules from `.github/copilot-instructions.md`.

## Change hygiene

- Prefer small, incremental changes.
- Avoid unrelated refactors and formatting.
- Keep public interfaces backward-compatible unless the request explicitly allows breaking changes.

## Design

- Prefer simple designs first; add abstraction only when justified.
- Keep functions small and cohesive.
- Fail fast with clear error messages.

## Testing

- When behavior changes, add or update tests.
- If tests exist, prefer running them (or proposing commands) before concluding.

## Documentation

- When you introduce new public behavior, update user-facing docs or README if relevant.
