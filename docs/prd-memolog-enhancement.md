# PRD: Editor Workflow Hardening for MemoLog

## Summary
MemoLog's highest-value near-term improvement is to make editing predictable, loss-resistant, and markdown-friendly for keyboard-heavy users. The shipped scope in this iteration focuses on editor workflow hardening rather than broad feature expansion.

## Problem
Users editing notes in the TUI hit several friction points:
- visual selection does not feel distinct enough from cursor-line highlighting
- nested markdown list editing is inconsistent, especially when exiting a list or continuing ordered items
- there is no obvious in-editor save flow comparable to Vim `:w`
- plain-text paste behavior can break list structure and duplicate markdown bullets
- shifted symbol input such as `~` can fail on some terminals/layouts
- right-side navigation panes feel unbalanced when focus moves away from the timeline
- Esc in Vim mode can trigger exit/discard behavior instead of acting purely like a Vim escape

## Users
- daily keyboard-first journaling users
- Vim-style editor users
- users capturing structured markdown lists/tasks
- users pasting external notes into MemoLog

## Goals
1. Reduce fear of losing edits.
2. Make list editing behavior feel intentional and markdown-native.
3. Improve visual clarity in selection and navigation layouts.
4. Preserve existing core workflows and keep the implementation small.

## Non-goals
- full markdown WYSIWYG rendering
- full command-line mode parity with desktop Vim
- a large redesign of timeline/agenda/tasks information architecture
- broad new data models or sync features

## Shipped Scope
### 1) Visual selection clarity
- selection highlight must win over cursor-line highlight
- selection color should be distinct from cursor-line color in built-in themes

### 2) Markdown editing ergonomics
- Enter on empty nested bullet/checklist items should reduce depth rather than trapping the user at the same level
- top-level empty bullets should exit the list cleanly
- ordered checklist continuation should increment correctly and normalize continuation text
- outdenting a top-level list item should remove the list marker instead of doing nothing

### 3) Save-in-editor behavior
- add a stay-in-editor save flow via `Ctrl+S`
- add Vim-style normal-mode `:w` support for in-place save
- saving an existing entry should keep the editing session open and mark the buffer clean

### 4) Plain-text paste and symbol input reliability
- bracketed paste/plain-text paste should insert raw text directly instead of replaying auto-indent logic line by line
- `~` input should work reliably in editor text input paths

### 5) Navigation balance
- when focus is on agenda/tasks, timeline and right column should use an even 50/50 width split

### 6) Vim Esc semantics
- in Vim editor mode, Esc in normal mode should clear pending commands only; it must not trigger discard/exit flow

## Acceptance Criteria
- visual selection highlight is visibly different from cursor-line highlight in built-in themes
- partial visual selections show selection color instead of cursor-line color where they overlap
- `Ctrl+S` saves without leaving the editor
- in Vim normal mode, `:w` saves without leaving the editor
- plain-text paste inserts the pasted text as-is in Editing/Search modes
- `~` can be typed in editing mode
- empty nested bullet + Enter outdents one level; empty top-level bullet + Enter exits the list
- focused agenda/tasks layout uses a balanced left/right split
- all changes pass `cargo fmt` and `cargo test`

## Risks
- save-in-place for brand-new entries must avoid duplicate entry creation on subsequent saves
- paste handling behavior depends on bracketed paste support from the terminal
- theme color changes should improve contrast without making selection visually noisy

## Follow-up Ideas
- richer ordered list / roman numeral / checkbox rendering
- visible command/status hint for `:w`
- explicit release notes/help popup updates for editor shortcuts
