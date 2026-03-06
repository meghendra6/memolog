# Design Rules: Editor Workflow Hardening

## Intent
Editing should feel safe, predictable, and lightweight. The UI should bias toward preserving flow for keyboard users.

## Rules
1. **Selection outranks cursor context**
   - if a cell is both selected and on the cursor line, show selection styling
   - selection and cursor-line colors must be visually distinguishable in bundled themes

2. **Never surprise-delete from Esc in Vim mode**
   - Esc is mode control, not a destructive navigation shortcut
   - Normal-mode Esc should reset transient edit state only

3. **Saving should preserve flow**
   - `Ctrl+S` and Vim `:w` should save in place and keep focus in the editor
   - a save should provide lightweight confirmation via toast, not a mode transition

4. **Paste must be literal-first**
   - pasted text should be inserted as plain text with normalized newlines
   - paste should bypass incremental key replay behavior that mutates markdown structure

5. **Markdown nesting should help the user exit structure**
   - empty nested bullets/checklists should outdent on Enter
   - empty top-level bullets should exit the list entirely
   - ordered checklist continuation should preserve structure and numbering

6. **Focused navigation should still feel balanced**
   - when agenda or tasks is focused, timeline and right column should split evenly
   - avoid overly collapsing context when focus shifts across the right-side panes

## Copy / Feedback
- save toast: `Saved. Continue editing.`
- delete-via-empty-save toast: `Entry deleted.`

## Accessibility / Ergonomics
- color distinction should not rely only on subtle hue shifts; use visibly different background values
- keyboard-first actions must avoid accidental context loss
