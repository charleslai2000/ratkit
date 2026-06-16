# MarkdownWidget Integration Guide

## Overview

`MarkdownWidget` is a terminal-based markdown renderer integrated into `ratkit`. It renders parsed markdown content with support for a table of contents (TOC) sidebar, line comments, scrollbar, statusline, vim-mode navigation, and syntax highlighting.

The widget is part of `ratkit::widgets::markdown_preview` and requires the `markdown-preview` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
markdown-preview = ["ratkit/markdown-preview"]
```

Without this flag, `MarkdownWidget` and all related types are not compiled.

---

## 2. Create All 12 Sub-States

`MarkdownWidget::new()` requires **every sub-state** to be passed at construction. There are no lazy-initialized defaults. Create each with `Default::default()` and pass them in the exact order the signature expects:

| Sub-State | Purpose |
|-----------|---------|
| `ScrollState` | Tracks scroll position and total line count |
| `SourceState` | Holds the raw markdown source string |
| `CacheState` | Caches rendered output for performance |
| `DisplaySettings` | Controls what is shown (line numbers, TOC, scrollbar, statusline) |
| `CollapseState` | Manages collapsed/expanded regions (headings, blocks) |
| `ExpandableState` | Tracks expandable sections |
| `GitStatsState` | Displays git blame/statistics if available |
| `VimState` | Enables vim-mode keybindings (h/j/k/l, gg, G, etc.) |
| `SelectionState` | Manages text selection within the rendered output |
| `DoubleClickState` | Detects double-click actions for copy/jump |

```rust
use ratkit::widgets::markdown_preview::{
    CacheState, CollapseState, DisplaySettings, DoubleClickState,
    ExpandableState, GitStatsState, MarkdownWidget, ScrollState,
    SelectionState, SourceState, VimState,
};

let mut scroll = ScrollState::default();
scroll.update_total_lines(markdown_content.lines().count().max(1));

let mut source = SourceState::default();
source.set_source_string(markdown_content);

let mut display = DisplaySettings::default();
display.set_show_document_line_numbers(true);

let widget = MarkdownWidget::new(
    markdown_content,
    scroll,
    source,
    CacheState::default(),
    display,
    CollapseState::default(),
    ExpandableState::default(),
    GitStatsState::default(),
    VimState::default(),
    SelectionState::default(),
    DoubleClickState::default(),
)
.with_has_pane(true)
.show_toc(true)
.show_scrollbar(true)
.show_statusline(true);
```

**Do not skip any state.** If you omit one, the widget will not render correctly or may panic.

---

## 3. Configure via Builder Methods

After construction, chain configuration methods on the widget handle. These must be called **before** entering the event loop:

| Method | Effect |
|--------|--------|
| `.with_has_pane(true)` | Enables pane-aware behavior (the widget knows it lives inside a pane) |
| `.show_toc(true)` | Enables the table of contents sidebar |
| `.show_scrollbar(true)` | Renders the scrollbar |
| `.show_statusline(true)` | Renders the status line at the bottom |
| `.with_frontmatter_collapsed(bool)` | Controls whether frontmatter is collapsed on load |
| `.set_line_comments(vec)` | Injects line-level annotations (comments, annotations) |

Calling these after construction has no effect.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. Your application struct must implement `CoordinatorApp`, which defines two methods:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events and must route them to the widget:

**Keyboard events:**
1. Extract `CrosstermKeyEvent` from the `CoordinatorEvent::Keyboard(key)` variant.
2. Pass it to `widget.handle_key(key_event)`.
3. Match the returned `MarkdownEvent` to handle application-level actions (e.g., clipboard operations, comment submission).
4. Return `CoordinatorAction::Redraw` when the widget emits a meaningful event; `Continue` otherwise.

**Mouse events:**
1. Extract `CrosstermMouseEvent` from the `CoordinatorEvent::Mouse(mouse)` variant.
2. Pass it to `widget.handle_mouse(event, area)` along with the widget's layout area (a `Rect`).
3. Match the returned event to handle clipboard, double-click, or TOC hover actions.

**Other events:**
- `CoordinatorEvent::Tick` — process timers, expired toast messages, or periodic tasks.
- `CoordinatorEvent::Resize` — always return `Redraw` to re-render the layout.
- `CoordinatorEvent::Other` — route as appropriate for your application.

### `on_draw(&mut self, frame: &mut Frame)`

This method receives a `ratatui::Frame` and is responsible for rendering:

1. Define your layout using `ratatui::Layout` (see Section 5).
2. Call `frame.render_widget(&mut self.widget, area)` where `area` is the widget's allocated region. The widget handles its own internal rendering — you do not render its children manually.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(1), Constraint::Min(0)])
    .split(frame.area());

let dev_bar_area = chunks[0];
let markdown_area = chunks[1];
```

Store `markdown_area` as a field on your app struct. You must pass it to `widget.handle_mouse()` on every mouse event. Without it, mouse coordinates are off and interactions fail silently.

---

## 6. Handle `MarkdownEvent` Variants

Both `widget.handle_key()` and `widget.handle_mouse()` return a `MarkdownEvent` enum. Match on its variants to trigger application-level behavior:

| Variant | Meaning | Required Action |
|---------|---------|-----------------|
| `MarkdownEvent::None` | No state change | Return `Continue` |
| `MarkdownEvent::Copied { text }` | User copied text | Populate the system clipboard yourself |
| `MarkdownEvent::CommentSubmitted { line, line_hash, line_text, comment_text }` | User submitted a line comment | Persist to your storage backend |
| `MarkdownEvent::TocHoverChanged { ... }` | TOC hover state changed | Trigger a `Redraw` to show hover tooltips |
| Other variants | State changed | Trigger a `Redraw` |

---

## 7. Coalesce Mouse-Move Events

Terminal emulators emit mouse-move events at 50–200 Hz. Without coalescing, the event queue saturates and the UI freezes.

Implement a timestamp-based filter:

```rust
use std::time::{Duration, Instant};

let mut last_move_processed = Instant::now();

// Inside on_event, for CoordinatorEvent::Mouse:
if matches!(mouse.kind, crossterm::event::MouseEventKind::Moved) {
    if self.last_move_processed.elapsed() < Duration::from_millis(24) {
        return Ok(CoordinatorAction::Continue); // Drop the event
    }
    self.last_move_processed = Instant::now();
}
```

Non-motion events (click, scroll) should always pass through — do not filter them.

---

## 8. Run the Application

Call the library's `run()` function with your app struct and a `RunnerConfig`:

```rust
use ratkit::prelude::{run, RunnerConfig};
use std::time::Duration;

let config = RunnerConfig {
    tick_rate: Duration::from_millis(250),
    ..RunnerConfig::default()
};
run(app, config)
```

This starts the event loop, layout engine, and render pipeline. The library handles the TUI lifecycle — you do not manage the event loop yourself.

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `markdown-preview` feature flag | Widget not compiled, build fails |
| 2 | Create all 12 sub-states with `Default::default()` | Widget panics or renders blank |
| 3 | Chain builder methods (`.show_toc()`, etc.) before event loop | Sidebar/statusline never appear |
| 4 | Store layout `Rect` from `Layout::split()` in app state | Mouse coordinates are wrong |
| 5 | Route `handle_key()` with `CrosstermKeyEvent` | Keyboard input ignored |
| 6 | Route `handle_mouse()` with `CrosstermMouseEvent` + area | Mouse clicks do nothing |
| 7 | Match `MarkdownEvent` variants from both handlers | Clipboard/comments broken |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Render with `frame.render_widget(&mut widget, area)` | Nothing renders on screen |
| 10 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::markdown_preview`:

- `MarkdownWidget` — The main widget struct
- `MarkdownEvent` — Enum of events emitted by `handle_key()` and `handle_mouse()`
- `ScrollState` — Scroll position and line count
- `SourceState` — Raw markdown source
- `CacheState` — Rendered output cache
- `DisplaySettings` — Visibility toggles (line numbers, TOC, scrollbar, statusline)
- `CollapseState` — Collapsed/expanded region tracking
- `ExpandableState` — Expandable section tracking
- `GitStatsState` — Git statistics display
- `VimState` — Vim-mode keybindings
- `SelectionState` — Text selection
- `DoubleClickState` — Double-click detection
- `MarkdownLineComment` — Line-level annotation struct (line, line_hash, line_text, comment_count, comment_text)
