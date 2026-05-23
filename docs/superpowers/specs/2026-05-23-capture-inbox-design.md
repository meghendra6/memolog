# Capture Inbox Design

## Summary
MemoLog should make quick capture frictionless while giving captured notes a clear path into later organization. This iteration adds a small, Markdown-native inbox workflow around Quick Capture and the existing search/timeline/editor tools.

## Problem
Quick Capture is fast, but the saved note immediately becomes one more timeline entry. If the user captures a rough thought, task seed, or follow-up item, there is no lightweight marker that says "review this later." The result is a split between fast capture and reliable organization.

## Goals
1. Preserve the speed of Quick Capture.
2. Make captured-but-unprocessed notes easy to find.
3. Reuse existing timeline, search, command palette, composer, and metadata tools.
4. Keep the data model plain Markdown and Obsidian-friendly.

## Non-goals
- Add a new dedicated review mode.
- Add hidden metadata or a separate local database for capture state.
- Auto-classify notes into tasks, schedules, or contexts.
- Remove `#inbox` automatically based on inferred completion.
- Change normal composer save behavior.

## Chosen Approach
Use an explicit `#inbox` tag as the capture marker.

Quick Capture entries receive `#inbox` by default unless the user already included it. The tag is visible in the Markdown file, searchable through existing search, and compatible with Obsidian and MemoLog's tag popup. A command palette action opens the inbox by applying the `#inbox` search query.

This is preferred over treating "metadata-free notes" as unprocessed because ordinary notes can intentionally remain unstructured. It is also preferred over hidden Quick Capture state because hidden state would weaken the plain-Markdown contract.

## User Flow
1. The user presses `Ctrl+N` to open Quick Capture.
2. The user types a short note and presses `Enter`.
3. MemoLog saves the note with `#inbox` appended if it is not already present.
4. MemoLog shows a toast such as `Quick note saved to #inbox.`
5. Later, the user opens the command palette and runs `Open inbox`.
6. MemoLog applies search query `#inbox` and focuses the timeline.
7. The user edits entries with existing composer tools:
   - `Ctrl+T` to turn a line into a task.
   - `Ctrl+P` to cycle priority.
   - `Ctrl+;` to add schedule, due date, time, or duration.
   - `Ctrl+W`, `Ctrl+E`, or `Ctrl+R` to set or clear context.
8. The user manually removes `#inbox` when the item is processed.

## UI Changes
### Quick Capture Popup
- Keep the popup compact.
- Update the help text to indicate that Quick Capture saves to `#inbox`.
- Keep `Enter` as save and `Esc` as cancel.

### Toast
- On successful Quick Capture save, show `Quick note saved to #inbox.`
- Error behavior remains unchanged.

### Command Palette
- Add `Open inbox`.
- Suggested aliases: `inbox`, `capture`, `review`.
- The action applies `#inbox` search and moves focus to the timeline.
- It should work even when there are no inbox entries; the empty search result is still useful feedback.

## Data Rules
- Only Quick Capture auto-appends `#inbox`.
- If the input already contains `#inbox`, do not append a duplicate.
- Do not append `#inbox` to regular composer saves.
- Preserve user text except for trimming the existing Quick Capture input and appending the marker when needed.
- The marker should be appended with a separating space.

Examples:

```text
call dentist
```

becomes:

```text
call dentist #inbox
```

```text
call dentist #inbox
```

stays:

```text
call dentist #inbox
```

## Implementation Boundaries
- Add a small helper for normalizing Quick Capture inbox content.
- Use that helper in the Quick Capture save path.
- Add a new command palette action and execution branch.
- Reuse existing search application behavior instead of adding new filtering state.
- Add focused tests for the helper and command palette behavior.
- Update README only where Quick Capture and search/review behavior are already described.

## Acceptance Criteria
- Quick Capture saves non-empty input with `#inbox` appended.
- Quick Capture does not duplicate `#inbox` if the user already typed it.
- Regular composer saves do not auto-append `#inbox`.
- Command Palette includes `Open inbox`.
- Running `Open inbox` applies the `#inbox` search query and focuses the timeline.
- Quick Capture popup copy communicates that entries go to `#inbox`.
- README documents the capture inbox flow briefly.
- `cargo test` passes.

## Risks
- Users may not want every Quick Capture entry to be an inbox item. This is acceptable for the first iteration because Quick Capture is specifically the low-friction capture path, while the regular composer remains unchanged.
- Manual `#inbox` removal is an extra step. This keeps the first version explicit and avoids ambiguous auto-completion rules.
- Search result behavior for an empty inbox must feel acceptable. The existing search UI should handle this without a new empty state.

## Future Ideas
- Add a single-key action to remove `#inbox` from the selected entry.
- Add a dedicated `Review inbox` workflow that walks entries one by one.
- Add optional config for the Quick Capture tag name.
- Add lightweight parser hints that suggest task/date/context actions while editing an inbox item.
