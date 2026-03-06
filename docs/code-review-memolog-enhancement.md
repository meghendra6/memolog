# Code Review: Editor Workflow Hardening

## Scope Reviewed
- editor command handling
- markdown continuation/outdent behavior
- paste event handling
- selection rendering precedence
- theme defaults for selection vs cursor line
- focused navigation layout split

## Main Findings
### Resolved
1. **Selection precedence ambiguity**
   - fixed by suppressing cursor-line background when a partial selection exists and by separating bundled theme colors.

2. **List exit behavior felt sticky**
   - fixed by allowing empty nested bullets/checklists to outdent and top-level bullets to exit.

3. **Missing save-in-place flow**
   - fixed with `Ctrl+S` plus Vim normal-mode `:w`.

4. **Paste path could distort markdown structure**
   - improved by handling `Event::Paste` as plain text insertion with newline normalization.

5. **Vim Esc semantics were overloaded**
   - normal-mode Esc now clears pending state instead of triggering discard flow.

## Residual Risks
- paste normalization depends on terminals emitting `Event::Paste`; non-bracketed paste may still arrive as key bursts
- multi-level ordered markdown support is still partial and should be treated as follow-up scope rather than complete parity

## Review Verdict
Approved for merge after formatting/tests pass.
