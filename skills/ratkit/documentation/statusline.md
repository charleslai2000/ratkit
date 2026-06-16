# Statusline Primitive Integration Guide

## Overview

`Statusline` is a one-row status-bar primitive integrated into `ratkit`. It comes in two flavors:

- `StatusLineStacked<'a>` — A PowerLine-style bar with stacked indicators on the left and right, plus a centered message. Uses the `SLANT_TL_BR` / `SLANT_BL_TR` glyphs.
- `StyledStatusLine<'a>` — A pre-configured Westinghouse-style reactor-control statusline with `OperationalMode` (Operational / Dire / Evacuate) and built-in render/event/metric counters.

The primitive is part of `ratkit::primitives::statusline` and requires the `statusline` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
statusline = ["ratkit/statusline"]
```

Without this flag, neither `StatusLineStacked` nor `StyledStatusLine` is compiled.

---

## 2. Construct a Statusline

### `StatusLineStacked`

```rust
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Span;
use ratkit::primitives::statusline::{StatusLineStacked, SLANT_BL_TR, SLANT_TL_BR};

let bar = StatusLineStacked::new()
    .start(
        Span::from(" STATUS ").style(Style::new().fg(Color::Black).bg(Color::DarkGray)),
        Span::from(SLANT_TL_BR).style(Style::new().fg(Color::DarkGray).bg(Color::Green)),
    )
    .center("Some status message...")
    .end(
        Span::from(" INFO ").style(Style::new().fg(Color::Black).bg(Color::Cyan)),
        Span::from(SLANT_BL_TR).style(Style::new().fg(Color::Cyan)),
    );
```

`StatusLineStacked` builder methods:

| Method | Effect |
|--------|--------|
| `.style(Style)` | Base style for the whole bar |
| `.start(text, gap)` | Append a `(text, gap)` pair to the left stack |
| `.start_bare(text)` | Append a single text to the left stack with no gap |
| `.end(text, gap)` | Append a `(text, gap)` pair to the right stack |
| `.end_bare(text)` | Append a single text to the right stack with no gap |
| `.center_margin(u16)` | Reserve a margin around the centered message |
| `.center(text)` | Set the centered message |

### `StyledStatusLine`

```rust
use ratkit::primitives::statusline::{OperationalMode, StyledStatusLine};

let bar = StyledStatusLine::new()
    .title(" RATKIT ")
    .mode(OperationalMode::Operational)
    .center_text("Runner loop active")
    .render_metrics(120, 1500)
    .event_metrics(80, 600)
    .message_count(42)
    .build();
```

`StyledStatusLine` builder methods:

| Method | Effect |
|--------|--------|
| `.title(&'a str)` | Prefix on the left side of the bar (default `" WESTINGHOUSE[STATUS]2 "`) |
| `.mode(OperationalMode)` | `Operational` (green), `Dire` (yellow), `Evacuate` (red) |
| `.center_text(impl Into<String>)` | Centered status text |
| `.render_metrics(count: usize, time_us: u64)` | Render counter and time in microseconds |
| `.event_metrics(count: usize, time_us: u64)` | Event counter and time in microseconds |
| `.message_count(u32)` | Pending message counter |
| `.use_slants(bool)` | Toggle PowerLine-style diagonal separators |
| `.build() -> impl Widget` | Materialize the widget (call after all setters) |

`OperationalMode` is `Copy + Eq` and defaults to `Operational`.

The two constants `SLANT_TL_BR = "\u{e0b8}"` and `SLANT_BL_TR = "\u{e0ba}"` are also re-exported from the module — they require a Nerd Font or PowerLine font in the terminal.

---

## 3. Configure via Builder Methods

Both builders are zero-cost — chain the setters above and call `.build()` (for `StyledStatusLine`) or render directly (for `StatusLineStacked`, which is already a `Widget`).

---

## 4. Integrate with `CoordinatorApp`

`Statusline` is a render-only primitive. The host owns the counters that feed it.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

- `CoordinatorEvent::Tick(_) => self.ticks += 1` — increment any counter that the statusline shows.
- `CoordinatorEvent::Keyboard` / `Mouse` — return `Redraw` when your app state changes.
- `CoordinatorEvent::Resize` — return `Redraw`.

The statusline itself does not consume events. It is a `Widget` and can be rendered with `frame.render_widget(bar, area)`.

### `on_draw(&mut self, frame: &mut Frame)`

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(1)])
    .split(frame.area());

let bar = StyledStatusLine::new()
    .title(" APP ")
    .mode(OperationalMode::Operational)
    .center_text("Ready")
    .render_metrics(self.renders, 1500)
    .event_metrics(self.events, 600)
    .message_count(self.pending as u32)
    .build();

frame.render_widget(bar, chunks[1]);
```

For `StatusLineStacked`, drop the `.build()` — it is already a widget:

```rust
let bar = StatusLineStacked::new()
    .start_bare(" APP ")
    .center("Ready")
    .end_bare(" q quit ");
frame.render_widget(bar, chunks[1]);
```

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`. Reserve a single row at the bottom (or top) of your layout for the bar:

```rust
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(1)])
    .split(frame.area());
```

The statusline is always exactly 1 row tall.

---

## 6. Update Counters Per Frame

The recommended pattern is to store `renders: usize` and `events: usize` on the app struct, increment them in `on_event` / `on_draw`, and rebuild the statusline from those values each frame:

```rust
impl CoordinatorApp for App {
    fn on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction> {
        self.events += 1;
        // ...
    }

    fn on_draw(&mut self, frame: &mut Frame) {
        self.renders += 1;
        let bar = StyledStatusLine::new()
            .title(" APP ")
            .render_metrics(self.renders, 0)
            .event_metrics(self.events, 0)
            .build();
        frame.render_widget(bar, ...);
    }
}
```

---

## 7. Coalesce Mouse-Move Events

The statusline does not consume events. If the rest of your app reacts to hover, coalesce at the host level:

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
| 1 | Enable `statusline` feature flag | Primitive not compiled, build fails |
| 2 | Pick `StatusLineStacked` (PowerLine) or `StyledStatusLine` (Westinghouse) | Wrong style for the look you want |
| 3 | Chain builder methods (`start`/`end`/`center` for stacked, or `mode`/`center_text`/`render_metrics` for styled) | Empty bar |
| 4 | Reserve `Constraint::Length(1)` at the top or bottom of your layout | Bar clipped or overlapping body |
| 5 | Increment render/event counters in `on_event` and `on_draw` | Counters always read zero |
| 6 | Call `.build()` for `StyledStatusLine` (the stacked variant is already a widget) | Compile error on `frame.render_widget` |
| 7 | Render with `frame.render_widget(bar, area)` | Nothing renders on screen |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::statusline`:

- `StatusLineStacked<'a>` — PowerLine-style stacked bar with `.style`, `.start`, `.start_bare`, `.end`, `.end_bare`, `.center_margin`, `.center`
- `StyledStatusLine<'a>` — Westinghouse-style bar with `.title`, `.mode`, `.center_text`, `.render_metrics`, `.event_metrics`, `.message_count`, `.use_slants`, `.build`
- `OperationalMode` — `Operational` (default), `Dire`, `Evacuate`
- `SLANT_TL_BR = "\u{e0b8}"`, `SLANT_BL_TR = "\u{e0ba}"` — PowerLine glyphs
