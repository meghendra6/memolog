---
name: execution-gating
description: Apply decision-based approval gating for risky actions (deps, migrations, destructive ops) while keeping low-risk edits fast. Keep changes safe without slowing routine coding.
---

# Execution Gating

<Critical_Principle>
**FAST FOR SAFE, STOP FOR RISKY**: Proceed immediately on LOW risk.
STOP and require approval on HIGH risk. No exceptions.
</Critical_Principle>

## Procedure

| Step | Action |
|------|--------|
| 1 | Classify action as LOW/MEDIUM/HIGH |
| 2 | If HIGH → STOP and request approval |
| 3 | Record risk level and rationale |

## Risk Classification

| Risk | Examples | Approval |
|------|----------|----------|
| **LOW** | Small localized edits, docs/tests, safe refactors | Not needed |
| **MEDIUM** | Multi-file changes, logic changes, critical paths | At decision points |
| **HIGH** | Dependencies, migrations, destructive ops, secrets | **REQUIRED** |

### HIGH Risk Actions (Always Require Approval)

| Category | Examples |
|----------|----------|
| Dependencies | Install/upgrade deps, lockfile changes |
| Build | Build tool changes, scaffolding regeneration |
| Database | Migrations, schema changes |
| Destructive | Mass delete/rename, sweeping refactors |
| System | Commands touching system state, secrets/tokens |
| Config | Broad auto-approve settings changes |

## Approval Protocol

When approval required, STOP and request:

```
Approval Needed: APPROVE: <action> / DENY: <reason>
```

## Safety Defaults

| Rule | Reason |
|------|--------|
| Prefer reversible changes | Easy rollback |
| Keep diffs small | Easier review |
| Preserve existing configs | Avoid breakage |
| Never expose secrets | Security |

## Output Template

### Execution Gate
- **Risk**: HIGH
- **Reason**: [specific reason]
- **Approval Needed**: `APPROVE: <action>` / `DENY: <reason>`
- **Safe Alternative**: [if applicable]
