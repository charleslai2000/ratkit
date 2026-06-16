# DocumentViewerWidget Integration Guide

## Overview

`DocumentViewerWidget` is the lower-level, stateless renderer integrated into `ratkit` that both `CodeWidget` and `MarkdownWidget` build on. It paints a normalized `RenderedDocument` (a `Vec<DocumentLine>` plus a `Vec<DocumentOutlineItem>`) with optional scrollbar, statusline, outline hover overlay, and selection highlighting. The viewer is the canonical "render a long document into a `Rect`" primitive.

The widget is part of `ratkit::widgets::document_viewer` and requires either the `code-widget` or `markdown-preview` feature flag to be enabled at compile time (this is the only module that is shared between them).

---

## 1. Feature Flag

The widget is shared between two higher-level widgets. Your `Cargo.toml` must enable one of them:

```toml
[features]
code-widget = ["ratkit/code-widget"]
# or
markdown-preview = ["ratkit/markdown-preview"]
```

Without either flag, `DocumentViewerWidget` and its state types are not compiled.

---

## 2. Build a Normalized Document

`DocumentViewerWidget` is stateless — every frame you give it a `RenderedDocument` to render. The higher-level widgets (`CodeWidget`, `MarkdownWidget`) build this for you; if you want to render arbitrary content directly:

```rust
use ratkit::widgets::document_viewer::{
    DocumentLine, DocumentLineKind, DocumentViewerWidget, RenderedDocument, DisplaySettings,
};

let document = RenderedDocument::new(
    vec![
        DocumentLine::new(1, vec![Span::raw("Hello, world!")], DocumentLineKind::Text),
        DocumentLine::new(2, vec![Span::raw("Section")], DocumentLineKind::Heading { level: 1, collapsed: false }),
    ],
    vec![], // outline
);

let display = DisplaySettings {
    show_line_numbers: false,
    show_scrollbar: true,
    ..DisplaySettings::default()
};
```

`DocumentLine` carries a `source_line: usize`, a list of styled `Span`s, and a `DocumentLineKind` (`Text`, `Heading { level, collapsed }`, `Code`, `Separator`, `Metadata`, `Empty`). `RenderedDocument::new(lines, outline)` is the only constructor you need.

---

## 3. Configure via Builder Methods

`DocumentViewerWidget` is built fresh per frame. Chain configuration methods on it before passing it to `viewer.render(area, buf, &scroll)`:

```rust
use ratatui::layout::Rect;
use ratatui::buffer::Buffer;

let viewer = DocumentViewerWidget::new(document, display)
    .selection(selection_state)
    .statusline("42%/100")
    .show_scrollbar(true)
    .outline_hover(false, None);

let area = Rect::new(0, 0, 80, 24);
let mut buf = frame.buffer_mut();
viewer.render(area, &mut buf, &scroll_state);
```

| Method | Effect |
|--------|--------|
| `.selection(SelectionState)` | Renders a highlighted range over the document |
| `.statusline(impl Into<String>)` | Draws a one-line statusline at the bottom of the area |
| `.show_scrollbar(bool)` | Toggles a vertical scrollbar on the right edge |
| `.outline_hover(hovered, entry)` | Shows a tooltip on the outline (TOC) overlay |

The widget does not own any state. You supply `&ScrollState` to `render()` so it knows which lines to show.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. The viewer is a low-level primitive — most users should pick `CodeWidget` or `MarkdownWidget` instead. If you do use it directly:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events. The viewer exposes two free functions for routing input:

| Function | Signature | Purpose |
|----------|-----------|---------|
| `handle_viewer_key` | `(KeyEvent, &mut ScrollState, &mut VimState) -> bool` | Vim-mode navigation: `j`/`k`, `gg`, `G`, `Ctrl+d`/`Ctrl+u`, page up/down |
| `handle_viewer_mouse` | `(MouseEvent, Rect, &mut ScrollState, &mut SelectionState) -> bool` | Scroll-wheel and click-to-select |

Route keyboard events to `handle_viewer_key` and mouse events to `handle_viewer_mouse`. Return `Redraw` when either function returns `true`.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue` unless you have a periodic refresh.
- `CoordinatorEvent::Resize` — always return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Define your layout with `ratatui::Layout`.
2. Build the `RenderedDocument` and the `DocumentViewerWidget` for this frame.
3. Call `viewer.render(area, buf, &scroll_state)` to paint into a buffer. The viewer is not a `ratatui::Widget`; it has an explicit `render(area, buf, scroll)` method.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(1)])
    .split(frame.area());

let document_area = chunks[0];
```

The viewer splits the area internally to fit an optional statusline. A `Min(0)` outer area is the simplest contract.

---

## 6. State Types (Reused by Code and Markdown Widgets)

`document_viewer` defines the shared state types that every long-document widget in `ratkit` re-uses. The list below maps the public exports to their purpose:

| Type | Purpose |
|------|---------|
| `DocumentLine` | One rendered line: `Vec<Span>` + `DocumentLineKind` |
| `DocumentLineKind` | `Plain`, `Heading { level }`, `Code`, `Quote`, `List`, `Divider`, `Frontmatter`, `Blank` |
| `DocumentOutlineItem` | One entry in the outline: `name`, `kind`, `line`, `level` |
| `RenderedDocument` | The pair `(Vec<DocumentLine>, Vec<DocumentOutlineItem>)` you pass to the viewer |
| `SelectionPos` | `x: i32, y: i32` (rendered-cell coordinates) used by `SelectionState` |
| `SourceState` | Source content and optional path |
| `ScrollState` | `current_line`, `total_lines`, `update_total_lines`, `ensure_current_visible` |
| `CacheState` | Render cache marker |
| `DisplaySettings` | `show_line_numbers`, `show_scrollbar`, `show_outline`, etc. |
| `SelectionState` | `active`, `anchor`, `cursor`, `frozen_lines`, `frozen_width`, `last_copied_text`; methods `clear`, `selected_range`, `enter`, `exit`, `select_line`, `extend_to` |
| `VimState` | Vim prefix state (e.g. `gg`) |
| `DocumentViewerWidget` | The renderer itself |
| `handle_viewer_key` | Free function: keyboard routing |
| `handle_viewer_mouse` | Free function: mouse routing |
| `render_document_lines` | Free function: paint lines into a `Rect` |

---

## 7. Coalesce Mouse-Move Events

If you wire mouse events yourself (not via `handle_viewer_mouse`), terminal emulators emit mouse-move events at 50–200 Hz. Coalesce them at the host level:

```rust
use std::time::{Duration, Instant};

let mut last_move_processed = Instant::now();

if matches!(mouse.kind, crossterm::event::MouseEventKind::Moved) {
    if self.last_move_processed.elapsed() < Duration::from_millis(24) {
        return Ok(CoordinatorAction::Continue);
    }
    self.last_move_processed = Instant::now();
}
```

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

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `code-widget` or `markdown-preview` feature flag | Viewer not compiled, build fails |
| 2 | Build a `RenderedDocument::new(lines, outline)` | Viewer panics or renders blank |
| 3 | Chain `.selection()`, `.statusline()`, `.show_scrollbar()`, `.outline_hover()` | Overlays never appear |
| 4 | Store layout `Rect` from `Layout::split()` in app state | Mouse coordinates are wrong |
| 5 | Route `handle_viewer_key()` with `CrosstermKeyEvent` | Keyboard navigation ignored |
| 6 | Route `handle_viewer_mouse()` with `CrosstermMouseEvent` + area | Selection and scroll-wheel broken |
| 7 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 8 | Render with `frame.render_widget(viewer, area)` | Nothing renders on screen |
| 9 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::document_viewer`:

- `DocumentViewerWidget` — The renderer (`Widget` + explicit `render(area, buf, scroll)`)
- `RenderedDocument` — `(lines, outline)` pair passed to the viewer
- `DocumentLine`, `DocumentLineKind`, `DocumentOutlineItem`, `SelectionPos`
- `SourceState`, `ScrollState`, `CacheState`, `DisplaySettings`
- `SelectionState`, `VimState`
- `handle_viewer_key`, `handle_viewer_mouse`, `render_document_lines` — Free input/render helpers
