# CodeWidget Integration Guide

## Overview

`CodeWidget` is a read-only source code viewer integrated into `ratkit`. It renders highlighted source lines with optional line numbers, a symbol outline overlay, scrollbar, statusline, vim-mode navigation, and copy/yank support. The widget is a thin `StatefulWidget` wrapper around the shared `DocumentViewerWidget` rendering pipeline.

The widget is part of `ratkit::widgets::code_widget` and requires the `code-widget` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
code-widget = ["ratkit/code-widget"]
```

Without this flag, `CodeWidget` and all related types are not compiled.

---

## 2. Create All Sub-States

`CodeWidget` is a `StatefulWidget` over `CodeState`. Create the state with `Default::default()` and seed the source:

```rust
use ratkit::widgets::{code_widget::{CodeState, CodeWidget}, document_viewer::SourceState};

let mut state = CodeState::default();

// Option A: load content from a string
state.source.set_source_string(rust_source);

// Option B: load from a file path
state.source.set_source_file("/path/to/file.rs")?;

// Optional: language override
state.set_language_override("rust");

// Optional: enable the outline and line numbers
state.display.show_outline = true;
state.display.show_line_numbers = true;
state.display.highlight_current_line = true;
```

`CodeState` bundles every sub-state the widget needs in one struct:

| Field | Purpose |
|-------|---------|
| `source: SourceState` | Raw source text and optional file path |
| `scroll: ScrollState` | Current line, total lines, ensure-visible |
| `selection: SelectionState` | Line range selection for yank |
| `vim: VimState` | Vim-mode prefix state (gg, G, etc.) |
| `display: DisplaySettings` | Outline, line numbers, highlight toggles |
| `cache: CacheState` | Render cache marker |
| `outline: Vec<DocumentOutlineItem>` | Last extracted symbol outline |
| `outline_hovered`, `outline_hovered_entry` | TOC hover state |
| `language_override: Option<String>` | State-level language override |
| `rendered_document: Option<RenderedDocument>` | Cached highlight result |

---

## 3. Configure via Builder Methods

`CodeWidget` is a per-frame renderer: you build it from state, chain display overrides, and render. Call these **each frame** before `frame.render_stateful_widget(...)`:

```rust
let widget = CodeWidget::from_state(&state)
    .show_line_numbers(true)
    .relative_line_numbers(false)
    .show_outline(true)
    .language("rust");
```

| Method | Effect |
|--------|--------|
| `.show_line_numbers(bool)` | Override line-number visibility for this render |
| `.relative_line_numbers(bool)` | Override relative line-number visibility |
| `.show_outline(bool)` | Override outline/TOC visibility |
| `.language("rust")` | Force a language for syntax detection on this render |

The widget falls back to `state.display` for any option you do not override on the builder.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. Your application struct must implement `CoordinatorApp`, which defines two methods:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events and must route them to the widget:

**Keyboard events:**
1. Extract the `crossterm::event::KeyEvent` from `CoordinatorEvent::Keyboard(keyboard)`.
2. Pass it to `widget.handle_key(key, &mut state)`.
3. Match the returned `CodeEvent` to drive application-level actions (`y` emits `Copied { text }` for clipboard).
4. Return `CoordinatorAction::Redraw` when the widget emits a meaningful event; `Continue` otherwise.

**Mouse events:**
1. Extract the `CrosstermMouseEvent` from `CoordinatorEvent::Mouse(mouse)`.
2. Pass it to `widget.handle_mouse(event, area, &mut state)` with the widget's layout `Rect`.
3. Match the returned event — `SelectionChanged` or `OutlineHoverChanged` — to trigger a `Redraw`.

**Other events:**
- `CoordinatorEvent::Tick` — increment your FPS counter, or recompute expensive state.
- `CoordinatorEvent::Resize` — always return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Define your layout with `ratatui::Layout` (see Section 5).
2. Call `frame.render_stateful_widget(self.widget(), self.code_area, &mut self.state)`. The widget handles its own internal rendering.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(1), Constraint::Min(0)])
    .split(frame.area());

self.code_area = chunks[1];
```

Store `code_area` as a field on your app struct. You must pass it to `widget.handle_mouse()` on every mouse event. Without it, outline hover and click coordinates are off and interactions fail silently.

---

## 6. Handle `CodeEvent` Variants

`widget.handle_key()` and `widget.handle_mouse()` both return a `CodeEvent`. Match on its variants:

| Variant | Meaning | Required Action |
|---------|---------|-----------------|
| `CodeEvent::None` | No state change | Return `Continue` |
| `CodeEvent::Navigated { line }` | Viewport focus moved to a line | Trigger `Redraw` |
| `CodeEvent::Copied { text }` | User yanked selection with `y` | Populate the system clipboard |
| `CodeEvent::SelectionChanged` | Mouse drag updated selection | Trigger `Redraw` |
| `CodeEvent::OutlineHoverChanged` | TOC hover state changed | Trigger `Redraw` to show tooltip |

`y` always copies the current `state.selection` text. Call `state.copy_selection()` to read the same text without consuming the event.

---

## 7. Coalesce Mouse-Move Events

Terminal emulators emit mouse-move events at 50–200 Hz. Without coalescing, the event queue saturates and the UI freezes.

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
| 1 | Enable `code-widget` feature flag | Widget not compiled, build fails |
| 2 | Build `CodeState::default()` and seed `source` | Widget panics or renders blank |
| 3 | Build `CodeWidget` each frame via `from_state` and chain overrides | Display toggles have no effect |
| 4 | Store layout `Rect` from `Layout::split()` in app state | Mouse coordinates are wrong |
| 5 | Route `handle_key()` with `CrosstermKeyEvent` | Keyboard input ignored |
| 6 | Route `handle_mouse()` with `CrosstermMouseEvent` + area | Mouse clicks and TOC hover do nothing |
| 7 | Match `CodeEvent` variants from both handlers | Clipboard and hover broken |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Render with `frame.render_stateful_widget(widget, area, &mut state)` | Nothing renders on screen |
| 10 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::code_widget` (with shared state types in `ratkit::widgets::document_viewer`):

- `CodeWidget` — `StatefulWidget` over `CodeState`
- `CodeState` — Bundles source, scroll, selection, vim, display, cache, outline
- `CodeEvent` — `None`, `Navigated { line }`, `Copied { text }`, `SelectionChanged`, `OutlineHoverChanged`
- `CodeLine` — Rendered source line with style
- `CodeLanguage` — Detected language descriptor
- `CodeOutlineItem` — Symbol outline entry (name, kind, line)
- `highlight_code_lines(text, lang)` — Highlight a string into `Vec<CodeLine>`
- `detect_language(path, text, override)` — Pick a `CodeLanguage` from path and content
- `extract_symbol_outline(text, lang)` — Produce a `Vec<CodeOutlineItem>`

Shared state types (re-exported via `document_viewer`):

- `SourceState`, `ScrollState`, `CacheState`, `DisplaySettings`
- `SelectionState`, `VimState`, `DocumentLine`, `DocumentLineKind`
- `DocumentOutlineItem`, `RenderedDocument`, `SelectionPos`
- `DocumentViewerWidget` — Lower-level renderer used internally
