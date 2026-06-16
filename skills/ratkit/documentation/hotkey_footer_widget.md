# HotkeyFooterWidget Integration Guide

## Overview

`HotkeyFooter` is a single-line footer widget integrated into `ratkit`. It renders an aerospace-tui style strip of `key description` pairs with configurable colors for keys, descriptions, and the background. It is the recommended "what can I press right now?" bar at the bottom of any TUI screen.

The widget is part of `ratkit::widgets::hotkey_footer` and requires the `hotkey-footer` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
hotkey-footer = ["ratkit/hotkey-footer"]
```

Without this flag, `HotkeyFooter` and `HotkeyItem` are not compiled.

---

## 2. Construct the Footer

`HotkeyFooter::new(items)` returns a footer with cyan keys, dark-gray descriptions, and a black background. Each item is a `HotkeyItem`:

```rust
use ratatui::style::Color;
use ratkit::widgets::hotkey_footer::{HotkeyFooter, HotkeyItem};

let footer = HotkeyFooter::new(vec![
    HotkeyItem::new("q", "quit"),
    HotkeyItem::new("?", "help"),
    HotkeyItem::new("/", "search"),
    HotkeyItem::new("Tab", "next"),
]);
```

`HotkeyItem::new(key, description)` accepts anything `Into<String>`. The `key` is rendered bold; the `description` is rendered in `description_color`.

---

## 3. Configure via Builder Methods

| Method | Effect |
|--------|--------|
| `.key_color(Color)` | Color for the key (default `Cyan`) |
| `.description_color(Color)` | Color for the description (default `DarkGray`) |
| `.background_color(Color)` | Background color of the entire strip (default `Black`) |
| `.with_theme_colors(key, desc, bg)` | Set all three colors at once |

There is no separate "selected" or "hover" state — the footer is decorative. Wire the matching logic into the host application's `on_event` and rebuild the footer on the fly when the available actions change:

```rust
let footer = HotkeyFooter::new(items)
    .key_color(Color::Yellow)
    .description_color(Color::White)
    .background_color(Color::Reset);
```

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. The footer is **not interactive** — it does not consume keyboard or mouse events. You only render it.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

- `CoordinatorEvent::Keyboard` — handle key bindings yourself, then return `Redraw` if the footer content should change.
- `CoordinatorEvent::Mouse` — ignore; forward to other widgets.
- `CoordinatorEvent::Resize` — return `Redraw`.
- `CoordinatorEvent::Tick` — return `Continue` unless you rotate the visible items over time.

### `on_draw(&mut self, frame: &mut Frame)`

There are two rendering paths:

```rust
// Option A: frame-based, for ratatui's Frame API
footer.render(frame, area);

// Option B: Widget trait, when rendering into a buffer
frame.render_widget(&footer, area);
```

`HotkeyFooter` implements `Widget` for both `&HotkeyFooter` and `HotkeyFooter`. The first form is convenient when you want to keep the footer owned by your app struct.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(1)])
    .split(frame.area());

let body_area = chunks[0];
let footer_area = chunks[1];
```

The footer is always exactly one row tall. Reserve `Constraint::Length(1)` at the bottom of your layout for it.

---

## 6. Update Items Per Screen

The footer is rebuilt on demand. The recommended pattern is to store a `Vec<HotkeyItem>` per "screen" or "mode" of your app and re-construct the footer in `on_draw`:

```rust
struct App {
    mode: Mode,
    footer: HotkeyFooter,
}

impl App {
    fn rebuild_footer(&mut self) {
        let items = match self.mode {
            Mode::Browse => vec![
                HotkeyItem::new("j/k", "move"),
                HotkeyItem::new("Enter", "open"),
                HotkeyItem::new("q", "quit"),
            ],
            Mode::Edit => vec![
                HotkeyItem::new("Esc", "back"),
                HotkeyItem::new("Ctrl+S", "save"),
            ],
        };
        self.footer = HotkeyFooter::new(items)
            .key_color(Color::Cyan)
            .description_color(Color::DarkGray);
    }
}
```

Call `rebuild_footer()` whenever `self.mode` changes, then return `Redraw` from `on_event`.

---

## 7. Coalesce Mouse-Move Events

The footer does not consume mouse events. Coalesce them at the host level if other widgets in the same app react to hover:

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
| 1 | Enable `hotkey-footer` feature flag | Widget not compiled, build fails |
| 2 | Construct with `HotkeyFooter::new(items)` | Build fails (no public default) |
| 3 | Chain `.key_color()` / `.description_color()` / `.background_color()` | Default colors only |
| 4 | Reserve `Constraint::Length(1)` at the bottom of your layout | Footer clipped or overlapping body |
| 5 | Rebuild footer when the available actions change | Footer shows stale hints |
| 6 | Render with `footer.render(frame, area)` or `frame.render_widget(&footer, area)` | Nothing renders on screen |
| 7 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 8 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::hotkey_footer`:

- `HotkeyFooter` — The footer widget. Holds `items`, `key_color`, `description_color`, `background_color`. Implements `Widget` for both `Self` and `&Self`. Exposes `.render(frame, area)` for the `Frame` API.
- `HotkeyItem` — A single `(key, description)` pair. `HotkeyItem::new(key, description)`.
