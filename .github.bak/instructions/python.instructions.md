---
name: Python Standards
description: Python-specific guidance for Copilot.
applyTo: "**/*.py"
---

# Python Standards

## Style & readability

- Follow PEP 8 conventions.
- Prefer explicit, readable code over cleverness.
- Use type hints for public functions and non-trivial internal functions.

## Structure

- Prefer small functions with clear inputs/outputs.
- Avoid global state; inject dependencies where practical.

## Error handling

- Raise specific exceptions with actionable messages.
- Avoid bare `except:`.

## Performance (practical)

- Avoid accidental quadratic loops in hot paths.
- Prefer vectorized operations when using numeric libraries.

## Comments

- Do not add verbose comments.
- Add docstrings only when they provide real value (public APIs, complex behavior).
