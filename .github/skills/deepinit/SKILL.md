---
name: deepinit
description: Create comprehensive, hierarchical AGENTS.md documentation across the codebase for AI agent context.
---

# Deep Init Skill

<Critical_Principle>
**AI-READABLE DOCUMENTATION**: Create AGENTS.md files that help AI agents understand the codebase structure, conventions, and relationships.
</Critical_Principle>

## Purpose

AGENTS.md files serve as AI-readable documentation that helps agents understand:
- What each directory contains
- How components relate to each other
- Special instructions for working in that area
- Dependencies and relationships

## Hierarchical Structure

Every AGENTS.md (except root) includes a parent reference:

```markdown
<!-- Parent: ../AGENTS.md -->
```

This creates navigable hierarchy:
```
/AGENTS.md                      ← Root (no parent tag)
├── src/AGENTS.md               ← <!-- Parent: ../AGENTS.md -->
│   ├── src/components/AGENTS.md
│   └── src/utils/AGENTS.md
└── docs/AGENTS.md
```

## AGENTS.md Template

```markdown
<!-- Parent: {relative_path}/AGENTS.md -->
<!-- Generated: {timestamp} -->

# {Directory Name}

## Purpose
{One-paragraph description of directory role}

## Key Files
| File | Description |
|------|-------------|
| `file.ts` | Brief description |

## Subdirectories
| Directory | Purpose |
|-----------|---------|
| `subdir/` | What it contains |

## For AI Agents

### Working In This Directory
{Special instructions for AI agents}

### Testing Requirements
{How to test changes}

### Common Patterns
{Code patterns used here}

## Dependencies

### Internal
{Other parts of codebase this depends on}

### External
{Key external packages}
```

## Procedure

| Step | Action |
|------|--------|
| 1 | Map directory structure (exclude node_modules, .git, dist, build) |
| 2 | Create work plan by depth level (parents before children) |
| 3 | Generate each AGENTS.md with proper parent reference |
| 4 | Validate hierarchy (all parent refs resolve) |

## When to Use

| Situation | Action |
|-----------|--------|
| New project setup | MANDATORY |
| Major restructuring | Recommended |
| Onboarding AI agents | Required |
| Directory purpose unclear | Required |

