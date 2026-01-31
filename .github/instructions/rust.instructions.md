---
name: Rust Standards
description: Rust-specific guidance for Copilot.
applyTo: "**/*.rs"
---

# Rust Standards

## Safety & correctness

- Avoid `unsafe` unless necessary and documented.
- Prefer `Result`/`Option` with `?` for propagation.
- Avoid `unwrap`/`expect` in production paths; add context on errors.

## Ownership & lifetimes

- Prefer borrowing over cloning; clone only when necessary.
- Keep lifetimes simple; avoid explicit lifetimes unless required.

## Style

- Follow `rustfmt` conventions.
- Prefer iterators and pattern matching for clarity.

## Testing

- Use `#[cfg(test)]` modules; keep tests focused.

## Comments

- Avoid verbose comments; document invariants or tricky lifetimes.
