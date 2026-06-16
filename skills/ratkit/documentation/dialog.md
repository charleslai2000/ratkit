# Dialog Primitive Integration Guide

## Overview

`Dialog` is a modal dialog primitive integrated into `ratkit`. It supports five dialog types (Info, Success, Warning, Error, Confirm), configurable buttons with keyboard navigation, a pluggable body renderer, backdrop, shadow, wrapping, modal/passthrough modes, and a customizable keymap. `DialogWidget` is the `ratatui::Widget` adapter that paints a `Dialog` into a buffer.

The primitive is part of `ratkit::primitives::dialog` and requires the `dialog` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
dialog = ["ratkit/dialog"]
```

Without this flag, `Dialog` and `DialogWidget` are not compiled.

---

## 2. Construct a Dialog

`Dialog::new(title, message)` and the typed constructors `info(...)`, `success(...)`, `warning(...)`, `error(...)`, and `confirm(...)` are the entry points. The default dialog is 60% × 40% of the screen, centered, with a `Tab`/`Right` next-button, `BackTab`/`Left` previous-button, `Enter` confirm, and `Esc` cancel keymap:

```rust
use crossterm::event::KeyCode;
use ratkit::primitives::dialog::{
    Dialog, DialogActionsLayout, DialogShadow, DialogWrap,
};

let dialog = Dialog::confirm("Delete file", "This cannot be undone.")
    .buttons(vec!["Yes", "No"])
    .default_selection(1)
    .actions_layout(DialogActionsLayout::Horizontal)
    .wrap_mode(DialogWrap::WordTrim)
    .shadow(DialogShadow::Medium)
    .next_keys(vec![KeyCode::Tab, KeyCode::Right])
    .previous_keys(vec![KeyCode::BackTab, KeyCode::Left])
    .wrap_button_navigation(true);
```

Public fields on `Dialog` include `title`, `message`, `dialog_type`, `buttons`, `selected_button`, `width_percent`, `height_percent`, `footer`, `footer_style`, `footer_alignment`, `title_inside`, `backdrop_style`, `shadow`, `modal_mode`, `border_color`, `style`, `button_selected_style`, `button_style`, `actions_layout`, `actions_alignment`, `message_alignment`, `content_padding`, `wrap`, `body_renderer`, `keymap`, `wrap_button_navigation`, `button_areas`, and per-`DialogType` theme color overrides.

---

## 3. Configure via Builder Methods

The builder methods come in two groups: simple setters and the typed keymap setters.

Simple setters:

| Method | Effect |
|--------|--------|
| `.dialog_type(DialogType)` | Set the dialog type (Info/Success/Warning/Error/Confirm) |
| `.buttons(Vec<&str>)` | Set the button labels |
| `.style(Style)` | Base content style |
| `.border_color(Color)` | Override the auto-colored border |
| `.width_percent(f32)`, `.height_percent(f32)` | Set size as a fraction of the parent area |
| `.footer(&str)` / `.hide_footer()` | Configure the optional footer text |
| `.title_inside(bool)` | Move the title inside the border |
| `.overlay(bool)` | Add a full-screen backdrop style |
| `.shadow(DialogShadow)` | Add a drop shadow (None, Soft, Medium, Strong, Custom) |
| `.modal_mode(DialogModalMode)` | `Blocking` (default) or `Passthrough` |
| `.actions_layout(DialogActionsLayout)` | `Horizontal` or `Vertical` |
| `.actions_alignment(Alignment)` | Align the button row |
| `.message_alignment(Alignment)` | Align the message text |
| `.content_padding(u16, u16)` | Set horizontal/vertical padding |
| `.wrap_mode(DialogWrap)` | `WordTrim`, `WordNoTrim`, or `NoWrap` |
| `.body_renderer(Box<dyn DialogBodyRenderer>)` | Plug in a custom body renderer |
| `.wrap_button_navigation(bool)` | Wrap around at the ends of the button list |
| `.default_selection(usize)` | Initial selected button index |
| `.next_keys(Vec<KeyCode>)`, `.previous_keys(Vec<KeyCode>)`, `.confirm_keys(Vec<KeyCode>)`, `.cancel_keys(Vec<KeyCode>)`, `.close_keys(Vec<KeyCode>)` | Per-direction keymap setters |

The keymap is exposed as a `DialogKeymap` struct with `next`, `previous`, `confirm`, `cancel`, `close` fields of `Vec<KeyCode>`. `DialogKeymap::default()` provides the standard Tab/BackTab/Enter/Esc mapping.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. `Dialog` owns all its state — you only need to route input and render.

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
1. Extract `KeyCode` from `CoordinatorEvent::Keyboard(keyboard)`.
2. Call `dialog.handle_key_event(key_code)` and match the returned `DialogEventResult`.
3. If the result was consumed and produced a `DialogAction::Confirm(idx)`, branch on `dialog.buttons[idx]`.
4. Return `Redraw` when state changes; `Continue` otherwise.

**Mouse events:**
- Call `dialog.handle_mouse_confirm(column, row)` to detect clicks on the button row. Returns the same `DialogEventResult` as the keyboard path.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue`.
- `CoordinatorEvent::Resize` — return `Redraw`. The dialog re-centers on the next render automatically.

### `on_draw(&mut self, frame: &mut Frame)`

Render the dialog last so it draws on top of your application content:

```rust
use ratkit::primitives::dialog::DialogWidget;

frame.render_widget(DialogWidget::new(&mut self.dialog), frame.area());
```

The widget clears the area, paints the backdrop (if any), then the centered dialog with its shadow.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The dialog sizes itself relative to the `area` you give it via `width_percent` and `height_percent`. The typical pattern is to pass `frame.area()` and let the dialog center itself:

```rust
let area = frame.area();
frame.render_widget(DialogWidget::new(&mut self.dialog), area);
```

For dialogs that embed application content (e.g. a list), set `.content_padding(0, 0)` and pass a body renderer — the dialog does not need a layout from the host.

---

## 6. Handle `DialogAction` Variants

`DialogEventResult.action` is one of:

| Variant | Meaning | Required Action |
|---------|---------|-----------------|
| `DialogAction::Select(usize)` | User moved selection with `next`/`previous` keys | Trigger `Redraw` |
| `DialogAction::Confirm(usize)` | User confirmed the currently selected button | Dispatch the action for `dialog.buttons[idx]` |
| `DialogAction::Cancel` | User pressed `Esc` or cancel key | Close the dialog, no-op the original intent |
| `DialogAction::Close` | User pressed a `close_keys` entry | Close the dialog (e.g. dismiss an info banner) |

`DialogEventResult::consumed(action)` and `DialogEventResult::ignored()` are constructors for the result type.

---

## 7. Plug in a Custom Body

Implement `DialogBodyRenderer` to draw arbitrary content inside the dialog frame:

```rust
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratkit::primitives::dialog::DialogBodyRenderer;

struct MyBody { /* state */ }

impl DialogBodyRenderer for MyBody {
    fn render_body(&mut self, area: Rect, buf: &mut Buffer) {
        // paint into `area`
    }
}

let dialog = dialog.body_renderer(Box::new(MyBody { /* ... */ }));
```

The dialog handles borders, padding, footer, and button row — the body renderer only fills the message area.

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
| 1 | Enable `dialog` feature flag | Primitive not compiled, build fails |
| 2 | Construct with `Dialog::new(...)` or `Dialog::confirm(...)` | Build fails (no public default) |
| 3 | Chain builder methods for buttons, shadow, keymap, body | Defaults applied; no shadow, no keymap changes |
| 4 | Store the `Dialog` on your app struct (mut) so `DialogWidget::new` can borrow it | Compile error when rendering |
| 5 | Route `handle_key_event()` for keyboard input | Keymap ignored |
| 6 | Route `handle_mouse_confirm()` for mouse clicks | Button clicks ignored |
| 7 | Match `DialogAction` variants to dispatch on confirm | Confirmations silently dropped |
| 8 | Render with `frame.render_widget(DialogWidget::new(...), area)` last | Dialog drawn under other widgets |
| 9 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::dialog`:

- `Dialog<'a>` — The dialog state struct
- `DialogWidget<'a, 'b>` — The `Widget` adapter; constructed with `DialogWidget::new(&mut Dialog)`
- `DialogType` — `Info`, `Success`, `Warning`, `Error`, `Confirm`
- `DialogActionsLayout` — `Horizontal`, `Vertical`
- `DialogWrap` — `WordTrim`, `WordNoTrim`, `NoWrap`
- `DialogModalMode` — `Blocking`, `Passthrough`
- `DialogShadow` — `None`, `Soft`, `Medium`, `Strong`, `Custom { offset_x, offset_y, style }`
- `DialogPadding`, `DialogFooter<'a>`, `DialogKeymap`, `DialogAction`, `DialogEventResult`
- `DialogBodyRenderer` — Trait with `fn render_body(&mut self, area: Rect, buf: &mut Buffer)`
- `DialogState` — Trivial `selected_button: usize` helper

Key methods on `Dialog`:

- `get_selected_button() -> usize`, `get_selected_button_text() -> Option<&str>`
- `get_border_color() -> Color`
- `select_next_button()`, `select_previous_button()`, `set_selected_button(usize)`
- `handle_click(col, row) -> Option<usize>`, `handle_key_event(KeyCode) -> DialogEventResult`, `handle_mouse_confirm(col, row) -> DialogEventResult`
- `blocks_background_events() -> bool`
