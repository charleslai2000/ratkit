# CodeDiffWidget Integration Guide

## Overview

`CodeDiff` is a stateful code-diff widget integrated into `ratkit`. It parses unified diff text into a structured `DiffFile` → `DiffHunk` → `DiffLine` model, supports both single-file focus and multi-file navigation, and renders to a ratatui `Buffer` via the `Widget` trait. Side-by-side row layout helpers and git-diff integration live alongside the widget for hosts that need them.

The widget is part of `ratkit::widgets::code_diff` and requires the `code-diff` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
code-diff = ["ratkit/code-diff"]
```

Without this flag, `CodeDiff` and all related types are not compiled.

---

## 2. Create the Widget

`CodeDiff::new()` returns an empty widget. The convenient constructors `from_unified_diff(&str)` and `from_files(Vec<DiffFile>)` parse diff input and seed the widget in one step:

```rust
use ratkit::widgets::code_diff::{CodeDiff, DiffConfig};

let diff = CodeDiff::from_unified_diff(
    "@@ -1,3 +1,3 @@\n-old line\n+new line\n unchanged\n",
)
.with_file_path("src/lib.rs")
.with_config(DiffConfig::new().with_show_line_numbers(true));
```

The widget owns the parsed `files: Vec<DiffFile>`, the `file_diffs: HashMap<String, Vec<DiffHunk>>` index, and the `hunks: Vec<DiffHunk>` view for the currently selected `file_path`. You do not pass hunks in directly — feed the widget a unified diff string or a pre-parsed file list.

---

## 3. Configure via Builder Methods

After construction, chain configuration methods on the widget handle. These are zero-cost and must be called **before** rendering:

| Method | Effect |
|--------|--------|
| `.with_file_path(path)` | Selects which file's hunks are exposed via `hunks()` / `hunks_mut()` |
| `.with_config(DiffConfig)` | Replaces the rendering configuration |
| `.add_hunk(DiffHunk)` | Appends a hunk to the currently selected file's hunks |

`DiffConfig` controls line-number visibility, added/removed/context colors, hunk header colors, and gutter width:

| Field | Default | Purpose |
|-------|---------|---------|
| `show_line_numbers` | `false` | Render line numbers in a left gutter |
| `added_fg`, `added_bg` | unset | Colors for `+` lines |
| `removed_fg`, `removed_bg` | unset | Colors for `-` lines |
| `context_fg`, `context_bg` | unset | Colors for unchanged context |
| `hunk_header_fg`, `hunk_header_bg` | unset | Colors for `@@` headers |
| `line_number_fg`, `gutter_width` | unset | Gutter styling and width |
| `context_lines` | `3` | Context lines around changes |

`CodeDiff` also implements `Default`, so `DiffConfig::default()` and `CodeDiff::default()` are valid fallbacks.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. Your application struct must implement `CoordinatorApp`, which defines two methods:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events. `CodeDiff` is a **read-only display widget** — it does not currently consume keyboard or mouse input. Route all events to your application logic and return `Redraw` to refresh the diff:

**Keyboard events:**
- Use `j`/`k` or `Tab` in your host code to cycle through `widget.files` and call `.with_file_path(...)` again.
- Return `CoordinatorAction::Quit` for `q`.

**Mouse events:**
- The widget renders into a `Rect` but does not own hit-testing. Forward clicks to your application logic.

**Other events:**
- `CoordinatorEvent::Tick` — refresh the diff by re-parsing from a source string and calling `CodeDiff::from_unified_diff(...)`.
- `CoordinatorEvent::Resize` — always return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Define your layout with `ratatui::Layout` (see Section 5).
2. Call `frame.render_widget(self.diff.clone(), area)`. The widget implements the `Widget` trait and renders a `Paragraph` of the form `Diff: <file_path>` for the selected file. The host application is responsible for richer rendering by walking `widget.hunks()` directly.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0)])
    .split(frame.area());

let diff_area = chunks[0];
```

Pass `diff_area` to `frame.render_widget(...)`. The widget will render into the entire area you give it.

---

## 6. Read the Parsed Model

The widget exposes the parsed diff model directly so the host can render custom views:

| Accessor | Returns |
|----------|---------|
| `widget.files` | `Vec<DiffFile>` — every file found in the unified diff |
| `widget.file_diffs` | `HashMap<String, Vec<DiffHunk>>` — file path to hunks |
| `widget.hunks()` | `&[DiffHunk]` — hunks for the currently selected file |
| `widget.hunks_mut()` | `&mut [DiffHunk]` — mutable view for the current file |
| `widget.file_path` | `Option<String>` — current focus |
| `widget.scroll_offset` | `usize` — host-managed first visible row |

`DiffFile` carries a `path: String` and `hunks: Vec<DiffHunk>`. `DiffHunk` carries an old/new line range, header text, and a `Vec<DiffLine>`. `DiffLine` is one of `Context`, `Added`, or `Removed` (via `ChangeType`/`DiffLineKind`) and may contain inline `DiffLineCell` segments for intra-line highlighting.

---

## 7. Pull Diff Text from Git

`get_git_diff(...)` in the `code_diff` module wraps `git diff` for the common "show me the working-tree diff" case. Use it as a convenience for refreshing the widget on a tick:

```rust
use ratkit::widgets::code_diff::get_git_diff;

if let Ok(text) = get_git_diff(".", None) {
    self.diff = CodeDiff::from_unified_diff(&text);
}
```

This is not part of the widget itself — it is a host-side helper exposed under the same feature gate.

---

## 8. Run the Application

Call the library's `run()` function with your app struct and a `RunnerConfig`:

```rust
use ratkit::prelude::{run, RunnerConfig};
use std::time::Duration;

let config = RunnerConfig {
    tick_rate: Duration::from_millis(500),
    ..RunnerConfig::default()
};
run(app, config)
```

This starts the event loop, layout engine, and render pipeline. The library handles the TUI lifecycle — you do not manage the event loop yourself.

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `code-diff` feature flag | Widget not compiled, build fails |
| 2 | Construct with `CodeDiff::new()` or `from_unified_diff(...)` | Widget renders nothing |
| 3 | Chain `.with_file_path(...)` and `.with_config(...)` before rendering | Wrong file displayed, no line numbers |
| 4 | Store layout `Rect` from `Layout::split()` in app state | Widget renders into wrong area |
| 5 | Iterate `widget.files` / `widget.hunks()` for richer output | Stuck on the default header-only render |
| 6 | Refresh via `from_unified_diff` on `Tick` or hotkey | Diff goes stale |
| 7 | Render with `frame.render_widget(self.diff.clone(), area)` | Nothing renders on screen |
| 8 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::code_diff`:

- `CodeDiff` — Stateful ratatui widget; implements `Widget`
- `DiffConfig` — Rendering configuration (colors, gutter, context)
- `DiffFile` — Parsed file entry with `path` and `hunks`
- `DiffFileStatus` — `Added`, `Deleted`, `Modified`, `Renamed`, `Unchanged`
- `DiffHunk` — One `@@` block with line ranges and `Vec<DiffLine>`
- `DiffLine` — One line of diff content with `kind` and inline cells
- `DiffLineCell` — Inline sub-segment for intra-line highlighting
- `DiffLineKind` — `Context`, `Added`, `Removed`
- `DiffStyle` — Rendering style preset for unified vs side-by-side
- `ChangeType` — `Equal`, `Insert`, `Delete`, `Replace` (mirrors `similar` crate)
- `InlineSegment` — Inline segment with text and style
- `SideBySideRow` — One side-by-side row with left/right `DiffLine`s
- `get_git_diff(...)` — Host-side helper that shells out to `git diff`
