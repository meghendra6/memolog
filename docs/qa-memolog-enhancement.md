# QA Report: Editor Workflow Hardening

## Validation Commands
- `cargo fmt`
- `cargo test -q`

## Result
- `cargo fmt`: PASS
- `cargo test -q`: PASS (`131 passed`)

## Focused Coverage
- visual selection precedence test in UI rendering
- markdown continuation/outdent tests for nested and top-level bullets
- ordered checklist continuation tests
- shifted symbol normalization test for `~`

## Manual Verification Targets
1. in Vim mode, press `v` and confirm the selected region is visually distinct from the cursor line
2. in Vim normal mode, run `:w` and confirm the editor stays open with a save toast
3. paste a nested bullet list and confirm it inserts as plain text without duplicated bullets
4. type `~` in the editor and confirm it appears
5. place the cursor on an empty nested bullet and press Enter; confirm outdent
6. move focus to agenda/tasks and confirm the layout splits evenly left/right

## Release Readiness
Ready to merge.
