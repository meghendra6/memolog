---
name: C/C++ Standards
description: C/C++-specific guidance for Copilot.
applyTo: "**/*.c,**/*.h,**/*.cc,**/*.cpp,**/*.cxx,**/*.hpp,**/*.hh,**/*.hxx"
---

# C/C++ Standards

## Safety & correctness

- Avoid undefined behavior; initialize variables and validate inputs.
- Use const-correctness to express intent.
- In C++, prefer RAII and smart pointers; avoid raw `new`/`delete`.

## Interfaces & headers

- Keep headers minimal; avoid heavy includes in headers.
- Prefer forward declarations where possible.
- Keep declarations and definitions consistent.

## Error handling

- In C, return explicit error codes and document ownership.
- In C++, follow existing project conventions (exceptions vs status returns).

## Performance (practical)

- Avoid unnecessary copies; pass by reference/pointer when appropriate.
- Be explicit about ownership and lifetimes.

## Comments

- Avoid verbose comments; explain non-obvious constraints or invariants.
