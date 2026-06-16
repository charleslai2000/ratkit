# ThemePickerWidget Integration Guide

## Overview

`ThemePicker` is a centered modal dialog integrated into `ratkit` for selecting from a built-in catalog of themes. It supports search/filter, live preview as you navigate, vim-style `j`/`k` navigation, and emits `Selected`, `Cancelled`, and `PreviewChanged` events. The 33 built-in themes are exposed as the `BUILTIN_THEMES` constant.

The widget is part of `ratkit::widgets::theme_picker` and requires the `theme-picker` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
theme-picker = ["ratkit/theme-picker"]
```

Without this flag, `ThemePicker` and all related types are not compiled.

---

## 2. Construct the Picker

`ThemePicker::new()` returns a hidden picker. Call `.show()` to open it:

```rust
use ratkit::widgets::theme_picker::{ThemePicker, ThemeColors};

let mut picker = ThemePicker::new();
picker.show();

// Optional: pre-seed the current preview
picker.set_current_theme(&ThemeColors::default());
picker.set_saved_index(0); // mark a theme as the saved default
```

The picker owns a `ThemePickerState` with `visible`, `index`, `filter`, `current_preview`, `saved_index`, and `original_index` fields. Use `picker.state()` / `picker.state_mut()` to read or mutate it directly.

---

## 3. Configure via Builder Methods

| Method | Effect |
|--------|--------|
| `.width(u16)` | Set popup width in cells (default `44`) |
| `.title(impl Into<String>)` | Set the popup title (default `Select Theme`) |
| `.show_footer(bool)` | Toggle the inline keybinding hint footer |
| `.show()` | Open the popup and snapshot the original saved theme |
| `.hide()` | Close the popup and clear the filter |
| `.is_visible()` / `.is_shown()` | Query the current open state |
| `.set_saved_index(usize)` | Mark a theme as the saved default (asterisk in the list) |
| `.set_current_theme(&ThemeColors)` | Seed the current preview |

`BUILTIN_THEMES: &[&str]` lists all 33 names. Use this constant when you need to drive the picker from your own UI.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. Your application struct must implement `CoordinatorApp`, which defines two methods:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events and must route them to the widget:

**Keyboard events:**
1. Extract `KeyCode` from `CoordinatorEvent::Keyboard(keyboard)`.
2. If the picker is visible, call `picker.handle_key(&key_code)` and match the returned `Option<ThemePickerEvent>`.
3. If the picker is hidden, treat your own toggle key (commonly `t`) as `picker.show()` / `picker.hide()`.
4. Return `CoordinatorAction::Redraw` when state changes; `Continue` otherwise.

**Mouse events:**
- The widget does not currently support mouse interaction. `picker.handle_mouse(...)` is a no-op stub. Forward mouse events to other widgets.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue`.
- `CoordinatorEvent::Resize` — always return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Render your normal background content first.
2. Call `picker.render(frame, frame.area())` *after* the background. The popup clears the underlying cells and draws on top, centered.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
let body = Paragraph::new("Press t to toggle the theme picker, q to quit")
    .block(Block::default().borders(Borders::ALL).title(" App "));
frame.render_widget(body, frame.area());
picker.render(frame, frame.area());
```

You typically pass `frame.area()` to `picker.render(...)` and let the widget center itself inside that rectangle.

---

## 6. Handle `ThemePickerEvent` Variants

`picker.handle_key()` returns `Option<ThemePickerEvent>`. Match on its variants:

| Variant | Meaning | Required Action |
|---------|---------|-----------------|
| `ThemePickerEvent::Selected(name)` | User pressed Enter on a theme | Apply `name` to your app theme; close the popup |
| `ThemePickerEvent::Cancelled` | User pressed Esc | Close the popup |
| `ThemePickerEvent::PreviewChanged(name)` | User navigated with `j`/`k` | Apply `name` as a live preview only — do not persist |

A typical host treats `PreviewChanged` as "apply the theme to the rest of the UI for visual feedback" and `Selected` as "commit the change to persistent storage".

---

## 7. Live Preview and State Snapshot

The picker keeps a `current_preview: ThemeColors` so the popup itself renders with the user's tentative selection. To keep the rest of your app in sync with the preview, subscribe to `PreviewChanged` and apply the same `ThemeColors` to your panes / menu bars / statuslines.

`ThemePickerStateSnapshot` is a serializable, plain-data snapshot of the picker state — useful if you need to round-trip the picker state through an external system (tests, IPC, undo/redo).

```rust
use ratkit::widgets::theme_picker::ThemePickerStateSnapshot;

let snap = ThemePickerStateSnapshot::from(picker.state());
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
| 1 | Enable `theme-picker` feature flag | Widget not compiled, build fails |
| 2 | Construct with `ThemePicker::new()` and call `.show()` to open | Popup is hidden by default |
| 3 | Chain `.width()`, `.title()`, `.show_footer()` to taste | Defaults applied, no failures |
| 4 | Route `handle_key()` for keyboard input when visible | Keyboard input ignored |
| 5 | Match `ThemePickerEvent` variants: `Selected`, `Cancelled`, `PreviewChanged` | Selection not persisted; no live preview |
| 6 | Render with `picker.render(frame, area)` *after* background | Popup renders under the background |
| 7 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 8 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::theme_picker`:

- `ThemePicker` — The main widget struct
- `ThemePickerEvent` — `Selected(String)`, `Cancelled`, `PreviewChanged(String)`
- `ThemePickerState` — `visible`, `index`, `filter`, `current_preview`, `saved_index`, `original_index`
- `ThemePickerStateSnapshot` — Plain-data snapshot for persistence
- `ThemeColors` — `primary`, `secondary`, `accent`, `background`, `background_menu`, `background_panel`, `text`, `text_muted`, `border`, `border_active`, `success`, `warning`, `error`, `info`
- `BUILTIN_THEMES: &[&str]` — The 33 built-in theme names (ayu, aura, carbonfox, catppuccin, …, zenburn)

Key methods on `ThemePicker`:

- `new()`, `width(u16)`, `title(impl Into<String>)`, `show_footer(bool)`
- `show()`, `hide()`, `is_visible()`, `is_shown()`
- `handle_key(&KeyCode) -> Option<ThemePickerEvent>`, `handle_mouse(MouseEvent)`
- `render(&mut self, frame: &mut Frame, area: Rect)`
- `set_saved_index(usize)`, `saved_index() -> usize`
- `set_current_theme(&ThemeColors)`, `state()`, `state_mut()`
