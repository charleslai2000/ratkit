# Pane Primitive Integration Guide

## Overview

`Pane` is a styled panel primitive integrated into `ratkit`. It wraps a `ratatui::widgets::Block` with a title, an optional icon prefix, padding around the content, an optional text footer in the border, and a configurable footer area for embedded widget footers. It renders arbitrary `Widget` content inside the bordered area.

The primitive is part of `ratkit::primitives::pane` and requires the `pane` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
pane = ["ratkit/pane"]
```

Without this flag, `Pane` is not compiled.

---

## 2. Construct a Pane

`Pane::new(title)` returns a pane with a rounded white border, a bold title, and zero padding on all sides. `Pane::default()` calls `Pane::new("Pane")`.

```rust
use ratatui::style::Color;
use ratatui::text::Line;
use ratkit::primitives::pane::Pane;

let pane = Pane::new("Output")
    .with_icon("▶")
    .with_uniform_padding(1)
    .border_style(ratatui::style::Style::default().fg(Color::Cyan));
```

Public fields on `Pane<'a>`: `title: String`, `icon: Option<String>`, `padding: (u16, u16, u16, u16)`, `text_footer: Option<Line<'a>>`, `footer_height: u16`, `border_style`, `border_type`, `title_style`, `footer_style`.

---

## 3. Configure via Builder Methods

| Method | Effect |
|--------|--------|
| `.with_icon(impl Into<String>)` | Prefix the title with an icon (e.g. `▶`, `■`) |
| `.with_padding(top, right, bottom, left)` | Asymmetric padding around the content |
| `.with_uniform_padding(u16)` | Set the same padding on all four sides |
| `.with_text_footer(Line<'a>)` | Render a `Line` inside the bottom border (e.g. status text) |
| `.with_footer_height(u16)` | Reserve an extra row band for a widget footer (used by `render_with_footer`) |
| `.border_style(Style)` | Style for the border lines |
| `.border_type(BorderType)` | `Plain`, `Rounded`, `Double`, `Thick`, etc. |
| `.title_style(Style)` | Style for the title text |
| `.footer_style(Style)` | Style for the text footer |
| `.with_theme(&AppTheme)` | Apply `theme.border` to `border_style` and `theme.text_muted` to `footer_style` (requires `markdown-preview`) |

---

## 4. Integrate with `CoordinatorApp`

`Pane` is a render helper. It does not consume events. You only call it from `on_draw`.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

- `CoordinatorEvent::Keyboard` / `Mouse` / `Tick` — return `Redraw` or `Continue` as needed for the rest of your app.
- `CoordinatorEvent::Resize` — return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

`Pane` exposes four render helpers on `Frame`:

| Method | Effect |
|--------|--------|
| `.render(frame, area, content)` | Render the pane frame and embed a `Widget` content |
| `.render_with_footer(frame, area, content, footer)` | Render the pane frame with a content widget and a footer widget split into a footer area of `footer_height` rows |
| `.render_paragraph(frame, area, lines: Vec<Line<'a>>)` | Render a `Paragraph` of `Line`s inside the pane (no extra widget needed) |
| `.render_paragraph_with_footer(frame, area, lines, footer)` | Same as above with a footer widget |
| `.render_block(area) -> (Rect, Option<Rect>)`, `.render_block_in_buffer(area, buf) -> (Rect, Option<Rect>)` | Render just the block and return `(inner, footer)` rects; useful when you want to draw content yourself |

```rust
pane.render_paragraph(
    frame,
    area,
    vec![
        Line::from("First line"),
        Line::from("Second line"),
    ],
);
```

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`. A typical vertical split for body and footer:

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(3)])
    .split(frame.area());

let body_area = chunks[0];
let footer_area = chunks[1];
```

Pass `body_area` to `pane.render(...)` and `footer_area` to whatever renders the footer.

---

## 6. Embed Any `Widget`

Every `render_*` method that takes a content argument accepts any `W: Widget`. That includes `Paragraph`, `List`, `Table`, `Tabs`, custom widgets, or one of the larger widgets in this crate (e.g. `DocumentViewerWidget`):

```rust
use ratatui::widgets::Paragraph;

pane.render(frame, area, Paragraph::new("Hello"));
```

`render_with_footer` is the right choice when the footer is more than a single text line — it gets its own row band inside the pane.

---

## 7. Coalesce Mouse-Move Events

`Pane` does not consume mouse events. If the rest of your app reacts to hover, coalesce at the host level:

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
| 1 | Enable `pane` feature flag | Primitive not compiled, build fails |
| 2 | Construct with `Pane::new(title)` | Build fails (no public default other than the one in `Default`) |
| 3 | Chain `.with_icon()`, `.with_uniform_padding()`, `.border_style()`, etc. | Default style only |
| 4 | Compute the pane `Rect` with `Layout::split(...)` | Pane renders into the wrong area |
| 5 | Render with `pane.render(...)` / `pane.render_paragraph(...)` / `pane.render_with_footer(...)` | Nothing renders on screen |
| 6 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 7 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::pane`:

- `Pane<'a>` — The pane struct with `title`, `icon`, `padding`, `text_footer`, `footer_height`, and styles
- `Pane::new(title)`, `with_icon(...)`, `with_padding(...)`, `with_uniform_padding(...)`, `with_text_footer(...)`, `with_footer_height(...)`, `border_style(...)`, `border_type(...)`, `title_style(...)`, `footer_style(...)`, `with_theme(&AppTheme)`
- `pane.render(frame, area, content)`, `pane.render_with_footer(frame, area, content, footer)`, `pane.render_paragraph(frame, area, lines)`, `pane.render_paragraph_with_footer(...)`, `pane.render_block(area) -> (Rect, Option<Rect>)`, `pane.render_block_in_buffer(area, buf) -> (Rect, Option<Rect>)`
