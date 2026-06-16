# Toast Primitive Integration Guide

## Overview

`Toast` is a transient notification primitive integrated into `ratkit`. It exposes a `Toast` value (message + level + duration), a `ToastManager` (collection with FIFO eviction and click-to-dismiss), the `ToastLevel` enum (`Success`, `Error`, `Info`, `Warning`) with `Color` and icon helpers, and a `render_toasts(frame, &manager)` free function that paints the active toasts in the lower-right corner of the screen.

The primitive is part of `ratkit::primitives::toast` and requires the `toast` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
toast = ["ratkit/toast"]
```

Without this flag, `Toast`, `ToastManager`, and `render_toasts` are not compiled.

---

## 2. Construct the Manager

`ToastManager::new()` returns a manager that holds up to 5 toasts by default. Toasts are added with `info(...)`, `success(...)`, `warning(...)`, `error(...)`, or a generic `add(Toast)`:

```rust
use ratkit::primitives::toast::{Toast, ToastLevel, ToastManager};

let mut toasts = ToastManager::new();
toasts.info("Background task finished");
toasts.error("Something went wrong");
toasts.warning("Careful now");
toasts.success("Saved");
```

`ToastManager` implements `Default` (3-second default duration, 5-slot cap).

`Toast::new(message, level, Option<Duration>)` and `Toast::with_duration(message, level, duration)` are the two constructors for a single toast. The default duration is `DEFAULT_TOAST_DURATION = 3 seconds`. `Toast::is_expired() -> bool` checks the live clock; `Toast::lifetime_percent() -> f32` returns the fraction of the toast's lifetime that is left (1.0 at creation, 0.0 at expiry).

`ToastLevel` exposes `color() -> ratatui::style::Color` and `icon() -> &'static str` helpers:

| Level | Color | Icon |
|-------|-------|------|
| `Success` | `Green` | `✓` |
| `Error` | `Red` | `✗` |
| `Info` | `Cyan` | `ℹ` |
| `Warning` | `Yellow` | `⚠` |

---

## 3. Configure via Manager Methods

| Method | Effect |
|--------|--------|
| `add(Toast)` | Push a toast (FIFO-evicts the oldest if over the cap) |
| `clear()` | Drop every toast |
| `remove_expired()` | Drop toasts whose duration has elapsed |
| `info(&str)`, `success(&str)`, `warning(&str)`, `error(&str)` | Convenience constructors with the matching `ToastLevel` |
| `get_active() -> &[Toast]` | All non-expired toasts |
| `has_toasts() -> bool` | True if there is at least one toast |
| `handle_click(col, row, frame_area) -> bool` | Click-to-dismiss; returns `true` if a toast was removed |

`render_toasts(frame, &manager)` is the free function that draws all active toasts. Each toast is rendered as a 40×3 rounded block in the lower-right corner of `frame.area()`, with a 2-cell margin and 1-cell spacing.

---

## 4. Integrate with `CoordinatorApp`

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
- Map your own keys to the level helpers: `t` for info, `e` for error, `c` for clear.

**Mouse events:**
1. Extract `column` and `row` from `CoordinatorEvent::Mouse(mouse)`.
2. Call `toasts.handle_click(column, row, frame.area())`.
3. If the call returns `true`, a toast was dismissed — return `Redraw`.

**Other events:**
- `CoordinatorEvent::Tick` — call `toasts.remove_expired()` to drop toasts whose time has elapsed, then return `Redraw`. The default duration is 3 seconds.
- `CoordinatorEvent::Resize` — return `Redraw`; the rendered position recomputes automatically.

### `on_draw(&mut self, frame: &mut Frame)`

Render toasts *last* so they sit on top of your application content:

```rust
use ratkit::primitives::toast::render_toasts;

// ... render your normal application into frame ...

render_toasts(frame, &self.toasts);
```

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. `render_toasts` reads `frame.area()` and positions toasts relative to it — it does not interact with your `Layout::split(...)` at all. To reserve space for toasts, increase the inner margin of any pane that you do not want overlapped, or accept that toasts float over the bottom-right corner.

---

## 6. Fire and Forget

The common pattern is to fire a toast and forget about it:

```rust
toasts.info("Build complete");
toasts.success(format!("Saved {} files", count));
toasts.error(format!("Failed: {}", err));
```

The manager handles expiration, FIFO eviction, and rendering. The host does not need to track per-toast state.

---

## 7. Coalesce Mouse-Move Events

Toasts do not respond to hover. Even so, coalesce at the host level to keep the event queue from saturating when other widgets react to move:

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

A 250 ms tick is fine for the default 3-second toast duration. If you use shorter toasts, lower the tick rate.

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `toast` feature flag | Primitive not compiled, build fails |
| 2 | Construct with `ToastManager::new()` | Build fails (no public default other than `Default`) |
| 3 | Call `info(...)` / `success(...)` / `warning(...)` / `error(...)` to fire toasts | No notifications appear |
| 4 | Call `remove_expired()` on every `Tick` | Toasts stay on screen after their duration |
| 5 | Render with `render_toasts(frame, &manager)` after the rest of the UI | Toasts hidden under other widgets |
| 6 | Route `handle_click` on left-button-down for click-to-dismiss | Clicks do nothing |
| 7 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 8 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::toast`:

- `Toast` — `message: String`, `level: ToastLevel`, `created_at: Instant`, `duration: Duration`
  - `new(message, level, Option<Duration>)`, `with_duration(message, level, duration)`
  - `is_expired() -> bool`, `lifetime_percent() -> f32`
- `ToastLevel` — `Success`, `Error`, `Info`, `Warning`
  - `color() -> ratatui::style::Color`, `icon() -> &'static str`
- `ToastManager` — FIFO collection, max 5 by default
  - `new()`, `add(Toast)`, `clear()`, `remove_expired()`
  - `info(...)`, `success(...)`, `warning(...)`, `error(...)`
  - `get_active() -> &[Toast]`, `has_toasts() -> bool`
  - `handle_click(col, row, frame_area) -> bool`
- `DEFAULT_TOAST_DURATION: Duration` — 3 seconds
- `render_toasts(frame: &mut Frame, manager: &ToastManager)` — Paint active toasts
