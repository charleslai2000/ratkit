# TermTUI Primitive Integration Guide

## Overview

`termtui` is a VT100 terminal emulation primitive integrated into `ratkit`. It exposes the same `vt100` parser and screen model that `mprocs` uses, plus a `render_screen` helper that paints a `Screen` into a ratatui `Buffer` and a `write_screen_diff` helper that emits the diff between two screen snapshots. This lets you embed a real PTY in your TUI without spawning a second terminal.

The primitive is part of `ratkit::primitives::termtui` and requires the `termtui` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
termtui = ["ratkit/termtui"]
```

Without this flag, the parser, screen, and diff helpers are not compiled.

---

## 2. The Public Surface

The module re-exports a large set of types from the `vt100` sub-module and adds two render helpers. Read the re-export list in `mod.rs` to see all of them; the most commonly used are:

| Type | Purpose |
|------|---------|
| `Parser` | Feed bytes from the PTY; produces `VtEvent`s and updates a `Screen` |
| `Screen` | Snapshot of the terminal grid: cursor position, cells, attributes |
| `Cell` | One cell: symbol + `Attrs` |
| `Attrs`, `Color` | Per-cell attributes (fg/bg + flags) and color |
| `Grid` | The full grid model |
| `Pos`, `Rect`, `Size`, `Margin`, `BorderType` | Geometry types |
| `VtEvent` | `Parser` events (e.g. bell, title, mouse, resize) |
| `MouseProtocolMode` | None / Sgr / X10 / Urxvt mouse tracking |
| `CursorStyle` | `Default`, `BlinkingBlock`, `SteadyBlock`, `BlinkingUnderline`, `SteadyUnderline`, `BlinkingBar`, `SteadyBar` |
| `ScreenDiffer` | Computes diffs between two `Screen`s |
| `BufferView` | Trait implemented by `Screen` for use with `ScreenDiffer` |
| `render_screen(&Screen, area, &mut Buffer)` | Paint a `Screen` into a `Buffer` |
| `write_screen_diff(&mut ScreenDiffer, &impl BufferView, &mut impl Write)` | Write the diff to any `Write` sink |

---

## 3. Typical Wiring

The recommended setup is:

1. Spawn a PTY (`portable_pty` or similar).
2. Spawn a reader thread that copies bytes from the master into a `Parser::new(screen, size)`.
3. Spawn a writer thread that pumps your `Frame` keys to the master.
4. On resize, call `parser.screen_mut().set_size(rows, cols)` and `pty.resize(rows, cols)`.
5. In `on_draw`, call `render_screen(&parser.screen(), area, frame.buffer_mut())`.

```rust
use ratatui::layout::Rect;
use ratatui::Frame;
use ratkit::primitives::termtui::{render_screen, Parser};

struct TerminalApp {
    parser: Parser,
}

impl CoordinatorApp for TerminalApp {
    fn on_event(&mut self, event: CoordinatorEvent) -> ratkit::LayoutResult<CoordinatorAction> {
        // forward keys to PTY master
        // forward mouse as needed
        Ok(CoordinatorAction::Redraw)
    }

    fn on_draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        render_screen(&self.parser.screen(), area, frame.buffer_mut());
    }
}
```

`render_screen` walks `(row, col)` over `area` and copies each `Screen` cell into the matching `Buffer` cell, converting `vt100::Color` → `ratatui::style::Color` and `Attrs` → `Style` along the way. Cells without contents render as a space; cells outside the screen render as `?`.

---

## 4. Integrate with `CoordinatorApp`

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
1. Extract `KeyEvent` from `CoordinatorEvent::Keyboard(keyboard)`.
2. Encode it as the appropriate terminal escape sequence (or write the raw byte) and write to the PTY master.
3. Return `Redraw`.

**Mouse events:**
1. If you want the embedded program to receive mouse, enable mouse tracking with the right `MouseProtocolMode` and forward `MouseEvent`s as escape sequences.
2. Return `Redraw` after each forwarded event.

**Other events:**
- `CoordinatorEvent::Tick` — pump any pending bytes from the PTY into `parser`, then `Redraw`.
- `CoordinatorEvent::Resize` — resize the PTY and `parser.screen_mut().set_size(rows, cols)`, then `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

The draw call is one line:

```rust
render_screen(&self.parser.screen(), area, frame.buffer_mut());
```

Optionally, frame the embedded screen with a `Block`:

```rust
use ratatui::widgets::{Block, Borders};

let block = Block::default().borders(Borders::ALL).title(" Shell ");
let inner = block.inner(area);
frame.render_widget(block, area);
render_screen(&self.parser.screen(), inner, frame.buffer_mut());
```

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`. The embedded screen occupies whatever `Rect` you give to `render_screen`. Most apps give it the entire `frame.area()` after subtracting header/footer rows:

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Length(1), Constraint::Min(0), Constraint::Length(1)])
    .split(frame.area());

// chunks[0] = dev bar, chunks[1] = terminal, chunks[2] = statusline
```

---

## 6. Differential Writes (for the writer thread)

If you want the writer thread to emit minimal ANSI escape sequences (e.g. when driving a remote terminal), use `ScreenDiffer` + `write_screen_diff`:

```rust
use std::io::Write;
use ratkit::primitives::termtui::{write_screen_diff, ScreenDiffer};

let mut differ = ScreenDiffer::new(prev_screen.into(), screen_size);
let mut sink = Vec::<u8>::new();
write_screen_diff(&mut differ, &screen, &mut sink)?;
sink_out.write_all(&sink)?;
```

`ScreenDiffer` tracks the previous snapshot and emits only the cells that changed. This is the same approach `mprocs` uses.

---

## 7. Coalesce Mouse-Move Events

The embedded program may or may not care about mouse moves. Even so, coalesce at the host level to keep the queue from saturating:

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
    tick_rate: Duration::from_millis(50), // pump PTY frequently
    ..RunnerConfig::default()
};
run(app, config)
```

A faster tick rate is important when the embedded program is chatty — the read thread sees bytes faster than the render thread, and you want a fresh `Screen` ready for the next frame.

---

## Integration Checklist

| # | Step | Failure Mode If Skipped |
|---|------|------------------------|
| 1 | Enable `termtui` feature flag | Primitive not compiled, build fails |
| 2 | Spawn a PTY and wire its master to a `Parser` | Nothing to render |
| 3 | Start a reader thread that calls `parser.process(&mut buf)` | Output never appears |
| 4 | Forward keyboard (and optionally mouse) to the PTY master | No input reaches the program |
| 5 | Resize the PTY and `parser.screen_mut().set_size(...)` on `Resize` | Embedded program runs at the wrong geometry |
| 6 | Render with `render_screen(&parser.screen(), area, frame.buffer_mut())` | Nothing renders on screen |
| 7 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 8 | Call `run(app, config)` with a small `tick_rate` | TUI loop runs but PTY output is choppy |

---

## Type Reference

All types live under `ratkit::primitives::termtui`:

- `Parser` — VT100 byte stream parser
- `Screen` — In-memory terminal screen state
- `VtEvent` — `Parser` events (title, bell, mouse mode, etc.)
- `Cell`, `Attrs`, `Color` — Cell-level data
- `Grid` — The cell grid
- `Pos`, `Rect`, `Size`, `Margin`, `BorderType` — Geometry types
- `MouseProtocolMode`, `CursorStyle` — Protocol enums
- `ScreenDiffer`, `BufferView` — Differential rendering primitives
- `render_screen(&Screen, Rect, &mut Buffer)` — Paint a screen
- `write_screen_diff(&mut ScreenDiffer, &impl BufferView, &mut impl Write)` — Emit the ANSI diff
