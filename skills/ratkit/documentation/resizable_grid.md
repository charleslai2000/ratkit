# ResizableGrid Primitive Integration Guide

## Overview

`ResizableGrid` is a split-pane layout primitive integrated into `ratkit`. It maintains a tree of `Split` and `Pane` nodes with stable `PaneId`s, supports horizontal/vertical splits, drag-to-resize dividers, and computed per-frame `PaneLayout` rects. `ResizableGridWidget` is the rendering and mouse-handling wrapper that turns a layout into a visible, interactive UI.

The primitive is part of `ratkit::primitives::resizable_grid` and requires the `resizable-grid` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
resizable-grid = ["ratkit/resizable-grid"]
```

Without this flag, `ResizableGrid` and `ResizableGridWidget` are not compiled.

---

## 2. Construct the Layout and Widget State

`ResizableGrid::new(pane_id)` starts with a single root pane. Use `split_pane_horizontally` and `split_pane_vertically` to grow the tree:

```rust
use ratkit::primitives::resizable_grid::{
    ResizableGrid, ResizableGridWidget, ResizableGridWidgetState,
};

let mut layout = ResizableGrid::new(0);
let bottom = layout.split_pane_horizontally(0).unwrap_or(0);
let _top_right = layout.split_pane_vertically(0).unwrap_or(0);
let _bottom_right = layout.split_pane_vertically(bottom).unwrap_or(0);

let _ = layout.resize_divider(0, 60);
let _ = layout.resize_divider(bottom, 40);

let widget_state = ResizableGridWidgetState::default();
```

`ResizableGrid` is plain data (`Clone`, `Default`). `ResizableGridWidgetState` is the host-side interaction state (`hovered_divider`, `dragging_divider`).

Constants and types exposed:

- `DEFAULT_SPLIT_PERCENT = 50`, `MIN_SPLIT_PERCENT = 10`, `MAX_SPLIT_PERCENT = 90`
- `SplitAxis::{Vertical, Horizontal}`
- `PaneId = u32` (stable identifiers)
- `LayoutNode::{Pane { id }, Split { axis, ratio, first, second }}`
- `SplitAreas` (computed left/right or top/bottom rects)
- `SplitDividerLayout` (`split_index`, `axis`, `area`, `ratio`)
- `PaneInfo` (per-pane metadata)

---

## 3. Configure via Builder Methods on the Widget

`ResizableGridWidget::new(layout)` returns a widget for the current frame. Chain display options before rendering:

```rust
let widget = ResizableGridWidget::new(layout.clone())
    .with_state(self.widget_state)
    .with_divider_style(...)
    .with_hover_style(...)
    .with_drag_style(...)
    .with_block(Block::default())
    .with_pane_borders(true);
```

| Method | Effect |
|--------|--------|
| `ResizableGridWidget::new(layout)` | Construct from a `ResizableGrid` |
| `.with_state(ResizableGridWidgetState)` | Replay prior interaction state (hover, drag) |
| `.state() -> ResizableGridWidgetState` | Read back the new state for the next frame |
| `.layout() -> &ResizableGrid`, `.layout_mut() -> &mut ResizableGrid` | Inspect or mutate the layout |
| `.with_divider_width(u16)` | Set the divider line width (min 1) |
| `.with_hit_threshold(u16)` | Set the hit-test threshold in cells (min 1) |
| `.with_hover_style(Style)` | Style for a hovered divider |
| `.with_drag_style(Style)` | Style for a divider being dragged |
| `.with_divider_style(Style)` | Style for normal pane borders |
| `.with_block(Block<'static>)` | Wrap the widget in a bordered block |
| `.with_pane_borders(bool)` | Toggle per-pane borders (default `true`) |
| `.optimal_poll_duration() -> Duration` | Recommended mouse-move poll interval (used for coalescing) |

The widget implements `Widget` and renders dividers + optional pane borders into the area you give it.

---

## 4. Integrate with `CoordinatorApp`

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
- The grid does not consume keys by default. Use your own keys to drive `layout.move_pane(...)` or `layout.remove_pane(...)`.

**Mouse events:**
1. Extract the `crossterm::event::MouseEvent` from `CoordinatorEvent::Mouse(mouse)`.
2. Build the widget for this frame with the latest `self.widget_state`:
   ```rust
   let mut widget = ResizableGridWidget::new(self.layout.clone())
       .with_state(self.widget_state);
   widget.handle_mouse(mouse, self.last_area);
   ```
3. Persist the resulting state and layout:
   ```rust
   self.widget_state = widget.state();
   self.layout = widget.layout().clone();
   ```
4. Return `Redraw` while the user is interacting, `Continue` otherwise.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue`.
- `CoordinatorEvent::Resize` — return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

```rust
let inner = block.inner(area);
self.last_area = inner;
let widget = ResizableGridWidget::new(self.layout.clone()).with_state(self.widget_state);
self.widget_state = widget.state();
self.layout = widget.layout().clone();
frame.render_widget(widget, inner);
```

Use `grid.layout_panes(area)` to get a `Vec<PaneLayout>` and render your own content into each pane. The widget itself only paints dividers and optional borders — your code paints the body of every pane.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The grid computes its own layout internally; the outer chrome is plain `ratatui`:

```rust
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders};

let block = Block::default().borders(Borders::ALL).title(" Grid ");
let inner = block.inner(frame.area());
frame.render_widget(block, frame.area());
```

Store `inner` (or the final rect) in `self.last_area` so `handle_mouse` and `render_widget` use the same coordinate system.

---

## 6. Layout Operations

| Method | Effect |
|--------|--------|
| `split_pane_vertically(pane_id) -> Option<PaneId>` | Split a pane into a left/right pair; returns the new pane id |
| `split_pane_horizontally(pane_id) -> Option<PaneId>` | Split a pane into a top/bottom pair |
| `resize_split(split_index, percent) -> bool` | Set a split's ratio (clamped to `MIN`/`MAX`) |
| `resize_divider(pane_id, percent) -> bool` | Resize the parent split of a pane |
| `move_pane(pane_id, target_pane_id) -> bool` | Move one pane to replace another |
| `remove_pane(pane_id) -> bool` | Remove a pane and collapse its split |
| `get_split_ratio(split_index) -> Option<u16>` | Read a split's current ratio |
| `set_split_percent(u16)` / `split_percent() / min_percent() / max_percent() -> u16` | Single-split read/write helpers |
| `layout_panes(area) -> Vec<PaneLayout>` | Compute per-pane rects for the given area |
| `layout_dividers(area) -> Vec<SplitDividerLayout>` | Compute per-divider rects for hit-testing |
| `calculate_split_area(area, percent) -> SplitAreas` | Compute left/right or top/bottom rects for a single split |

The `PaneLayout` result type carries `pane_id()` and `area()`.

---

## 7. Coalesce Mouse-Move Events

Resizable grids generate many drag events. Coalesce at the host level using the widget's `optimal_poll_duration()`:

```rust
use std::time::{Duration, Instant};

let mut last_move_processed = Instant::now();
let poll = Duration::from_millis(24);

if matches!(mouse.kind, crossterm::event::MouseEventKind::Moved) {
    if self.last_move_processed.elapsed() < poll {
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

This starts the event loop, layout engine, and render pipeline. The library handles the TUI lifecycle — you do not manage the event loop yourself.

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `resizable-grid` feature flag | Primitive not compiled, build fails |
| 2 | Construct `ResizableGrid::new(...)` and call `split_pane_*` | Layout has one pane only |
| 3 | Pair with a `ResizableGridWidgetState::default()` | Hover and drag state lost between frames |
| 4 | Store `last_area` rect from your `Layout::split(...)` | Mouse coordinates are off |
| 5 | Rebuild the widget each frame and call `handle_mouse` for mouse events | Drag/hover broken |
| 6 | Persist `widget.state()` and `widget.layout()` back to your app struct | Drag resets on the next frame |
| 7 | Walk `grid.layout_panes(area)` to render content per pane | Panes are empty |
| 8 | Render with `frame.render_widget(widget, area)` | Nothing renders on screen |
| 9 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 10 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::resizable_grid`:

- `ResizableGrid` — `root_index`, `nodes: Vec<LayoutNode>`, `next_pane_id`, `hovered_split`, `dragging_split`, `hit_threshold`
- `ResizableGridWidget` — The render/mouse wrapper
- `ResizableGridWidgetState` — `hovered_divider: Option<usize>`, `dragging_divider: Option<usize>`
- `LayoutNode`, `SplitAxis`, `SplitDirection` (deprecated alias), `SplitAreas`, `SplitDividerLayout`
- `PaneId = u32`, `PaneInfo`, `PaneLayout`
- `DEFAULT_SPLIT_PERCENT = 50`, `MIN_SPLIT_PERCENT = 10`, `MAX_SPLIT_PERCENT = 90`
