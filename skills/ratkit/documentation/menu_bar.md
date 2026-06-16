# MenuBar Primitive Integration Guide

## Overview

`MenuBar` is a horizontal menu bar primitive integrated into `ratkit`. It renders a rounded-border row of `MenuItem` entries separated by ` │ ` dividers, with distinct styles for normal, selected, hover, and selected+hover states. Items may carry an icon, a numeric value, and an optional one-shot action callback.

The primitive is part of `ratkit::primitives::menu_bar` and requires the `menu-bar` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
menu-bar = ["ratkit/menu-bar"]
```

Without this flag, `MenuBar` and `MenuItem` are not compiled.

---

## 2. Construct a Menu Bar

`MenuBar::new(items)` returns a bar with default normal/selected/hover/selected-hover styles. The bar tracks an `area: Option<Rect>` and a per-item `area: Option<Rect>` for hit-testing.

```rust
use ratkit::primitives::menu_bar::{MenuBar, MenuItem};

let menu = MenuBar::new(vec![
    MenuItem::new("File", 0),
    MenuItem::with_icon("Edit", "✎", 1),
    MenuItem::new("View", 2),
])
.with_selected(0);
```

`MenuItem` constructors:

| Constructor | Effect |
|-------------|--------|
| `MenuItem::new(name, value)` | Plain item |
| `MenuItem::with_icon(name, icon, value)` | Item with a leading icon string |
| `MenuItem::with_action(name, value, \|| { ... })` | Item that fires a one-shot closure on click |
| `MenuItem::with_icon_and_action(name, icon, value, \|| { ... })` | Icon + action |

Public fields on `MenuItem`: `name`, `icon`, `value: usize`, `selected`, `hovered`, `area`, `action`. `display_label()` returns the rendered text (icon + name) — useful for width measurement.

---

## 3. Configure via Builder Methods

| Method | Effect |
|--------|--------|
| `.with_selected(usize)` | Pre-select an item |
| `.normal_style(Style)` | Style for unselected, unhovered items |
| `.selected_style(Style)` | Style for the selected, unhovered item |
| `.hover_style(Style)` | Style for unselected, hovered items |
| `.selected_hover_style(Style)` | Style for the selected, hovered item |
| `.with_theme(&AppTheme)` | Set all four styles from a theme (requires the `theme` feature) |
| `.apply_theme(&AppTheme)` | Same as `with_theme` but in-place |

`MenuBar` also implements `Default` (one item named `"Menu Item"`).

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. The bar owns its items and the host owns hit-testing and selection state.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
- The bar does not consume keys by itself. Use your own keys (`Left`/`Right` to cycle, `Enter` to fire the action) and mutate `menu.items[i].selected` directly. Use `menu.selected() -> Option<usize>` to read the current selection.

**Mouse events:**
1. Extract `column` and `row` from `CoordinatorEvent::Mouse(mouse)`.
2. On `MouseEventKind::Moved`, call `menu.update_hover(column, row)` to mark the hovered item.
3. On `MouseEventKind::Down(MouseButton::Left)`, call `menu.handle_mouse(column, row) -> WidgetEvent`. Match on `WidgetEvent::MenuSelected { index, action }` — the closure in `action` is consumed, so wire the side effect here.
4. Return `Redraw` on any change.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue`.
- `CoordinatorEvent::Resize` — return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

The bar exposes three render methods, all on `Frame`:

| Method | Effect |
|--------|--------|
| `menu.render(frame, area)` | Render flush-left into the area |
| `menu.render_with_offset(frame, area, left_offset)` | Render with a fixed left margin |
| `menu.render_centered(frame, area)` | Compute the needed width and center the bar |

`MenuBar` is a borrowed-mut helper; the item `area` fields are updated as a side effect, so `update_hover` and `handle_mouse` work on the next event.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`. A typical 3-row menu bar at the top of the screen:

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(3), Constraint::Min(0)])
    .split(frame.area());

self.menu.render(frame, chunks[0]);
```

The bar draws its own rounded border, so reserve `Constraint::Length(3)` (1 row top border + 1 row content + 1 row bottom border).

---

## 6. Action Callbacks

`MenuItem::with_action` and `with_icon_and_action` accept a `Box<dyn FnOnce() + Send>`. When the user clicks the item, `handle_mouse` returns `WidgetEvent::MenuSelected { index, action }` with the closure. You invoke the action yourself:

```rust
match menu.handle_mouse(mouse.column, mouse.row) {
    WidgetEvent::MenuSelected { index, action } => {
        if let Some(cb) = action {
            cb();
        }
        self.last_selected = Some(index);
    }
    _ => {}
}
```

The closure runs on the event-loop thread, so keep it short or dispatch to a worker.

---

## 7. Coalesce Mouse-Move Events

`MenuBar` updates hover state on every move. With many items this is fine, but coalesce at the host level to keep the event queue from saturating:

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

This starts the event loop, layout engine, and render pipeline. The library handles the TUI lifecycle — you do not manage the event loop yourself.

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `menu-bar` feature flag | Primitive not compiled, build fails |
| 2 | Construct with `MenuBar::new(items)` | Build fails (no public default) |
| 3 | Chain `.with_selected(...)` and the four `*_style(...)` setters | Default styles only |
| 4 | Store the `MenuBar` on your app struct (mut) | Compile error when rendering |
| 5 | Route `update_hover()` and `handle_mouse()` for mouse events | Hover and click detection broken |
| 6 | Run `MenuItem::with_action` closures in response to `MenuSelected` | Clicks do nothing |
| 7 | Render with `menu.render(frame, area)` (or `render_centered` / `render_with_offset`) | Nothing renders on screen |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::menu_bar`:

- `MenuBar` — The bar with `items`, `area`, and the four style fields
- `MenuItem` — `name`, `icon`, `value: usize`, `selected`, `hovered`, `area`, `action: Option<Box<dyn FnOnce() + Send>>`
- `display_width(&str) -> usize` — Utility for width measurement in cell units

Key methods on `MenuBar`:

- `new(items)`, `with_selected(usize)`, `with_theme(&AppTheme)`, `apply_theme(&AppTheme)`
- `normal_style(...)`, `selected_style(...)`, `hover_style(...)`, `selected_hover_style(...)`
- `update_hover(col, row)`, `handle_click(col, row) -> Option<usize>`, `handle_mouse(col, row) -> WidgetEvent`
- `selected() -> Option<usize>`, `render(frame, area)`, `render_with_offset(frame, area, left_offset)`, `render_centered(frame, area)`
