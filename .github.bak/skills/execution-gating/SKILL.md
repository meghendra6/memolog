---
name: execution-gating
description: Apply decision-based approval gating for risky actions (deps, migrations, destructive ops) while keeping low-risk edits fast. Use this to keep changes safe without slowing down routine coding.
---

# Execution Gating (Decision-Based)

## Goal
Keep execution fast for LOW risk changes, but require explicit approval for actions that can destabilize the repo or environment.

## Procedure
1) Classify the action as LOW, MEDIUM, or HIGH using the risk list below.
2) If HIGH, stop and request approval using the protocol.
3) Record the risk level and rationale in the response.

## Risk guide
### LOW (no approval needed)
- Small localized edits
- Docs/tests additions
- Refactors that do not change public APIs or behavior significantly

### MEDIUM (approval at decision points)
- Multi-file changes
- Significant logic changes
- Touching critical paths (auth, payments, infra code)

### HIGH (approval required before executing)
- Dependency installs/upgrades; lockfile churn
- Build tool changes; project scaffolding regeneration
- DB migrations / schema changes
- Destructive ops (mass delete/rename), sweeping refactors
- Commands that touch system state or require secrets/tokens
- Broad auto-approve settings changes

## Approval protocol
When approval is required, stop and request:
- `APPROVE: <action>`
- or `DENY: <reason>`

## Safety defaults
- Prefer reversible changes and small diffs.
- Preserve existing configs unless explicitly requested.
- Never expose secrets; never ask to paste tokens into chat.

## Output template (when gating triggers)
### Execution Gate
Risk: HIGH
Reason: ...
Approval Needed: `APPROVE: <action>` / `DENY: <reason>`
Safe Alternative: ...
