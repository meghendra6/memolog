---
name: TypeScript Standards
description: TypeScript/JavaScript-specific guidance for Copilot.
applyTo: "**/*.ts,**/*.tsx,**/*.js,**/*.jsx"
---

# TypeScript Standards

## Type safety

- Prefer precise types over `any`.
- Keep public APIs well-typed and documented.
- Prefer `unknown` over `any` when input shape is not guaranteed.

## Code style

- Keep functions small and cohesive.
- Avoid unnecessary indirection.
- Preserve existing lint/format conventions.

## React (if applicable)

- Prefer functional components and hooks.
- Keep components focused; split when responsibilities grow.

## Comments

- Avoid excessive comments. Only explain non-obvious intent.
