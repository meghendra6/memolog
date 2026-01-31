---
name: security-review
description: Run a comprehensive security review on code changes
argument-hint: "Scope of security review (files/features to review)"
agent: SecurityReviewer
tools: ["textSearch", "codebase", "changes", "runInTerminal"]
---

# Security Review Prompt

Review the current code changes for security vulnerabilities.

## Scope

Focus on:

1. Recent changes (git diff)
2. Files handling user input
3. Authentication/authorization code
4. API endpoints
5. Database queries

## Checklist

### CRITICAL

- [ ] No hardcoded secrets (API keys, passwords, tokens)
- [ ] No SQL injection vulnerabilities
- [ ] No command injection risks

### HIGH

- [ ] All user input validated
- [ ] XSS prevention in place
- [ ] CSRF protection enabled
- [ ] Authentication required where needed

### MEDIUM

- [ ] Dependencies up to date (run appropriate audit)
- [ ] Error messages don't leak sensitive info
- [ ] Rate limiting on public endpoints

## Dependency Audit by Stack

| Stack   | Command                       |
| ------- | ----------------------------- |
| Node.js | `npm audit` or `pnpm audit`   |
| Python  | `pip-audit` or `safety check` |
| Go      | `govulncheck ./...`           |
| Rust    | `cargo audit`                 |

## Secret Detection

Search for potential secrets using `#tool:textSearch` with patterns like:

- `api.key`, `api_key`, `apikey`
- `password`, `secret`, `token`
- Hardcoded URLs with credentials

## Output

Provide a security review report with:

- Severity-rated findings
- Specific file:line references
- Remediation suggestions
- APPROVE / REQUEST CHANGES verdict
