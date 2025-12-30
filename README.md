# MemoLog

MemoLog is a terminal-based daily memo + task logger that writes to plain Markdown files.

This project was forked from https://github.com/sonohoshi/sonomemo.

## What it does

- **Timeline**: browse and edit timestamped log entries (multi-line supported)
- **Tasks**: detect Markdown checkboxes (`- [ ]`, `- [x]`) and toggle them
- **Pomodoro per task**: start a timer for a selected task; when it completes, MemoLog appends `🍅` to that task line
- **Search / Tags**: find entries across days
- **Markdown rendering**: lists (multi-level), checkboxes, headings, inline code, code fences, links, tags
- **Vim-first TUI**: focus switching + navigation optimized for tmux splits

## Install

### From crates.io

`cargo install memolog`

### From source

```bash
git clone https://github.com/meghendra6/sonomemo.git memolog
cd memolog
cargo install --path .
```

## Run

`memolog`

## Data model

- Logs are stored as `YYYY-MM-DD.md` files under `data.log_path`.
- Each log entry is a timestamped block:
  - First line: `[HH:MM:SS] <your first line>`
  - Following lines: stored as-is (no auto prefix insertion)
- App state is stored at `<log_path>/.memolog/state.toml` (carryover bookkeeping, etc.)

## Configuration

MemoLog loads `config.toml` from the OS config directory by default.

### Environment variables

- `MEMOLOG_CONFIG`: override config file path
- `MEMOLOG_DATA_DIR`: override default data directory
- `MEMOLOG_LOG_DIR`: override default log directory (used as default `data.log_path`)

### Example

The repository root also includes a small `config.toml` you can copy and edit.

### Theme

You can customize the UI colors by adding a `[theme]` section to `config.toml`.
Colors accept the built-in names (case-insensitive) or RGB values in `R,G,B` form.
If `[theme]` is omitted, MemoLog uses a theme preset (see `[ui] theme_preset`).

```toml
[theme]
border_default = "Blue"
border_editing = "Cyan"
border_search = "LightBlue"
border_todo_header = "Cyan"
text_highlight = "0,0,100"
todo_done = "LightGreen"
todo_wip = "Magenta"
tag = "Cyan"
mood = "Blue"
timestamp = "LightCyan"
```

Theme presets can be selected via config or the Theme Switcher popup.

```toml
[ui]
theme_preset = "Dracula Dark"
```

Available presets:
- Dracula Dark
- Solarized Dark
- Solarized Light
- Nord Calm
- Mono Contrast

## Keybindings (defaults)

All keybindings are configurable in `config.toml`.

Global:
- `?` help
- `h`/`left` focus timeline, `l`/`right` focus tasks
- `i` compose, `Tab`/`Shift+Tab` focus next/prev
- `q` quick capture, `/` search, `t` tags, `a` activity
- `o` log dir, `p` pomodoro, `N` jump to Now
- `T` theme presets, `V` editor style, `Ctrl+Q` quit

Timeline:
- `j`/`k` or `down`/`up` move, `Ctrl+d`/`Ctrl+u` page
- `g` top, `G` bottom
- `Enter`/`Space` toggle checkbox
- `e` edit entry, `Delete`/`x` delete entry

Tasks:
- `j`/`k` or `down`/`up` move
- `Enter`/`Space` toggle checkbox
- `p` pomodoro, `e` edit source entry
- `n` mark/unmark Now, `f` filter toggle, `1/2/3` open/done/all

Composer:
- `Ctrl+S` save, `Enter` newline
- `Tab`/`Shift+Tab` indent/outdent
- `Esc` back, `Ctrl+L` clear

Search:
- `Enter` apply, `Esc` cancel, `Ctrl+L` clear

Popups:
- `Enter`/`y` confirm, `Esc`/`n` cancel
- `j`/`k` or `down`/`up` move

Quick Capture popup:
- `Enter` save, `Ctrl+Enter` save & continue, `Esc` cancel

Vim mode: the composer editor uses Vim-like normal/insert/visual motions in addition to the bindings above.

## License

MIT. See `LICENSE`.
