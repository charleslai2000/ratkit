# Button Primitive Integration Guide

## Overview

`Button` is a tiny hover-aware label primitive integrated into `ratkit`. It produces a `[ Label ]` `Span` with a tracked bounding box so the host can detect hover and click. `render_title_with_buttons(...)` stacks multiple buttons on the right edge of a title bar.

The primitive is part of `ratkit::primitives::button` and requires the `button` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
button = ["ratkit/button"]
```

Without this flag, `Button` and `render_title_with_buttons` are not compiled.

---

## 2. Construct a Button

`Button::new(label)` returns a button with cyan bold text in the normal state and black-on-cyan bold text on hover. Use `.normal_style(...)` and `.hover_style(...)` to override:

```rust
use ratatui::style::{Color, Modifier, Style};
use ratkit::primitives::button::Button;

let mut button = Button::new("Save")
    .normal_style(Style::default().fg(Color::White))
    .hover_style(
        Style::default()
            .fg(Color::Black)
            .bg(Color::Green)
            .add_modifier(Modifier::BOLD),
    );
```

Fields are crate-internal (`pub(crate)`): `text: String`, `area: Option<Rect>`, `hovered: bool`, `normal_style: Style`, `hover_style: Style`. Read them through the public accessors `.text()`, `.area()`, `.hovered()`, `.hover()`, `.normal()`.

`Button` also implements `Default` (text `"Button"`). `Button::text()` and `Button::area()` return the current text and last-rendered bounding box, respectively.

---

## 3. Configure via Builder Methods

| Method | Effect |
|--------|--------|
| `.normal_style(Style)` | Style when not hovered |
| `.hover_style(Style)` | Style when hovered |
| `.set_area(Rect)` | Manually set the bounding box (usually set by `.render_*`) |
| `.render(panel_area, title_prefix) -> (Span, Rect)` | Render into a panel area, returns the span and bounding box |
| `.render_at_offset(panel_area, offset_from_right) -> (Span, Rect)` | Render with a manual offset from the right edge |
| `.render_with_title(panel_area, title) -> Line` | Render and combine with a title line into a `Line<'static>` |
| `.is_clicked(col, row) -> bool` | Hit-test against the stored area |
| `.update_hover(col, row)` | Update `hovered` based on the latest mouse position |
| `.hovered() -> bool`, `.hover() -> Style`, `.normal() -> Style` | Read state and styles |
| `.text() -> &str`, `.area() -> Option<Rect>` | Read state |

`render_title_with_buttons(panel_area, title, &mut [buttons])` produces a `Line` with the title on the left and the buttons stacked on the right.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. The button itself is not a `Widget` — it produces spans that you embed in a `Paragraph` or `Line`. The host owns hit-testing.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
- Buttons are mouse-driven. Map your own keys (e.g. `Enter` on the focused button) to action handling.

**Mouse events:**
1. Extract `column` and `row` from `CoordinatorEvent::Mouse(mouse)`.
2. For every button, call `button.update_hover(column, row)`.
3. On `MouseEventKind::Down(MouseButton::Left)`, call `button.is_clicked(column, row)` to detect a click.
4. Return `Redraw` when any state changes; `Continue` otherwise.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue`.
- `CoordinatorEvent::Resize` — return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Compute the area you want the button to live in.
2. Call `button.set_area(area)` (or use `.render(...)` / `.render_with_title(...)` to derive it).
3. Build a `Paragraph` from the span and call `frame.render_widget(...)`.

```rust
let span = if self.button.hovered() {
    Span::styled(format!(" [{}] ", self.button.text()), self.button.hover())
} else {
    Span::styled(format!(" [{}] ", self.button.text()), self.button.normal())
};
let paragraph = Paragraph::new(Line::from(span));
frame.render_widget(paragraph, area);
```

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(1), Constraint::Min(0)])
    .split(frame.area());

let title_bar = chunks[0];
let body = chunks[1];
```

Buttons are 1 row tall and `format!(" [{}] ", text).len() + 1` cells wide. Compute the bounding box at render time and store it on the button.

---

## 6. Combine with Other Primitives

`render_title_with_buttons` is the standard way to put one or more buttons into a title bar:

```rust
use ratkit::primitives::button::render_title_with_buttons;

let title_line = render_title_with_buttons(
    panel_area,
    "Settings",
    &mut [&mut save_button, &mut cancel_button],
);
let paragraph = Paragraph::new(title_line);
frame.render_widget(paragraph, panel_area);
```

The function mutates each button to record its bounding box, so `update_hover` and `is_clicked` will work without manual `set_area` calls.

---

## 7. Coalesce Mouse-Move Events

Terminal emulators emit mouse-move events at 50–200 Hz. Hover state is the only thing the button updates on move, so coalescing matters when many buttons are on screen:

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
| 1 | Enable `button` feature flag | Primitive not compiled, build fails |
| 2 | Construct with `Button::new(text)` | Build fails (no public default) |
| 3 | Chain `.normal_style()` / `.hover_style()` if needed | Default colors only |
| 4 | Set the bounding box with `set_area` or `render_*` | `is_clicked` always returns `false` |
| 5 | Route `update_hover` for every mouse-move | Hover state never updates |
| 6 | Route `is_clicked` for left-button-down | Clicks never fire |
| 7 | Render the produced `Span` / `Line` in a `Paragraph` | Nothing renders on screen |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::button`:

- `Button` — The button struct with `text`, `area`, `hovered`, `normal_style`, `hover_style`
- `render_title_with_buttons(area, title, &mut [&mut Button]) -> Line<'static>` — Render a title bar with stacked right-aligned buttons
