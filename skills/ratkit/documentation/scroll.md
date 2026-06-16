# Scroll Primitive Integration Guide

## Overview

`scroll` is a single-function utility module integrated into `ratkit`. It exposes `calculate_scroll_offset(selected_index, visible_count, total_count) -> usize`, a generic algorithm that keeps a selected item visible and centered in a scrollable viewport. Use it from any custom scrollable list, table, or tree you build on top of `ratatui`.

The primitive is part of `ratkit::primitives::scroll` and requires the `scroll` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
scroll = ["ratkit/scroll"]
```

Without this flag, `calculate_scroll_offset` is not compiled.

---

## 2. The Single Function

`calculate_scroll_offset` is the entire public surface of the module:

```rust
pub fn calculate_scroll_offset(
    selected_index: usize,
    visible_count: usize,
    total_count: usize,
) -> usize
```

The algorithm:

- If all items fit in the viewport, returns `0`.
- If the selected item is in the first half of the viewport, scroll to the top (`0`).
- If the selected item is in the last half of the viewport, scroll so the last items are visible (`total_count - visible_count`).
- Otherwise, center the selected item: `selected_index - half_visible`.

```rust
use ratkit::primitives::scroll::calculate_scroll_offset;

let offset = calculate_scroll_offset(10, 5, 20);
assert_eq!(offset, 8);
```

---

## 3. Worked Examples

| Call | Result | Why |
|------|--------|-----|
| `calculate_scroll_offset(5, 10, 10)` | `0` | All items fit; no scroll needed |
| `calculate_scroll_offset(2, 5, 20)` | `0` | Selection in first half; scroll to top |
| `calculate_scroll_offset(18, 5, 20)` | `15` | Selection in last half; show last 5 items |
| `calculate_scroll_offset(10, 5, 20)` | `8` | Selection in middle; center it |
| `calculate_scroll_offset(0, 1, 1)` | `0` | Edge case: single item |
| `calculate_scroll_offset(0, 1, 100)` | `0` | Edge case: first item |
| `calculate_scroll_offset(99, 1, 100)` | `99` | Edge case: last item, single visible slot |
| `calculate_scroll_offset(0, 3, 10)` | `0` | `half_visible = 1`; selection at `0` is in the first half |
| `calculate_scroll_offset(2, 3, 10)` | `1` | Selection just past the first half |
| `calculate_scroll_offset(6, 3, 10)` | `5` | Last-half branch fires |

The function never panics and never returns a value larger than `total_count.saturating_sub(visible_count)`.

---

## 4. Integrate with `CoordinatorApp`

`scroll` is a pure function — it has no state, no events, no rendering. The host application calls it once per draw, in `on_draw`.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

`scroll` does not consume events. Use your own keys to mutate `self.selected` (e.g. `j`/`k` or `Down`/`Up`):

```rust
match keyboard.key_code {
    KeyCode::Down => self.selected = (self.selected + 1).min(self.items.len() - 1),
    KeyCode::Up => self.selected = self.selected.saturating_sub(1),
    _ => {}
}
```

`CoordinatorEvent::Resize` should return `Redraw` so the visible count and offset recompute.

### `on_draw(&mut self, frame: &mut Frame)`

1. Split the body area with `ratatui::Layout`.
2. Compute the visible count from the area height.
3. Call `calculate_scroll_offset(self.selected, visible_count, self.items.len())`.
4. `skip(offset).take(visible_count)` over your items and render the slice.

```rust
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Borders, Paragraph};
use ratkit::primitives::scroll::calculate_scroll_offset;

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(3)])
    .split(frame.area());

let visible_count = chunks[0].height.saturating_sub(2) as usize;
let offset = calculate_scroll_offset(self.selected, visible_count.max(1), self.items.len());

let lines: Vec<Line> = self
    .items
    .iter()
    .enumerate()
    .skip(offset)
    .take(visible_count)
    .map(|(idx, item)| {
        if idx == self.selected {
            Line::from(format!("> {}", item))
        } else {
            Line::from(format!("  {}", item))
        }
    })
    .collect();

let paragraph = Paragraph::new(lines).block(Block::default().borders(Borders::ALL).title(" Items "));
frame.render_widget(paragraph, chunks[0]);
```

---

## 5. Layout Is Pure `ratatui`

`scroll` is layout-agnostic. It takes a `visible_count` and a `total_count` and returns an offset. Pair it with whatever `ratatui` layout you already use.

---

## 6. Combine with Other Primitives

The offset returned by `calculate_scroll_offset` is just a number. Pass it to any consumer that needs it:

- A `Paragraph::new(lines).scroll((offset as u16, 0))` (ratatui built-in vertical scroll)
- A `List::new(items)` with `ListState::default().with_offset(offset)`
- A custom `TreeView` that walks `state.expanded` and slices the result
- A `Table` whose row source is `items.iter().skip(offset).take(visible_count)`

The function is intentionally decoupled from rendering so the host can choose.

---

## 7. Coalesce Mouse-Move Events

`scroll` does not consume events, but if the rest of your app reacts to hover, coalesce at the host level:

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

Scroll-wheel events (`MouseEventKind::ScrollUp` / `ScrollDown`) should always pass through — they are the natural way to drive `self.selected` up and down.

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
| 1 | Enable `scroll` feature flag | Function not compiled, build fails |
| 2 | Track `self.selected: usize` and `self.items: Vec<T>` in your app state | Function has nothing to compute against |
| 3 | Update `self.selected` from keyboard and scroll-wheel events | Selection does not move |
| 4 | Compute `visible_count` from the body's inner height | Wrong slice rendered |
| 5 | Call `calculate_scroll_offset(self.selected, visible_count, self.items.len())` each frame | Items snap to top, selection scrolls out of view |
| 6 | Apply the offset with `.skip(offset).take(visible_count)` | Rendering works, but selection jumps around |
| 7 | Render with `Paragraph` / `List` / `Table` of the sliced items | Nothing renders on screen |
| 8 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::scroll`:

- `calculate_scroll_offset(selected_index: usize, visible_count: usize, total_count: usize) -> usize` — The single public function. Returns the scroll offset to apply so the selected item is visible and centered.
