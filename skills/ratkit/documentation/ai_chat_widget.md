# AIChatWidget Integration Guide

## Overview

`AIChat` is an interactive chat widget integrated into `ratkit`. It renders a multi-line message history with role-based styling, a multi-line input editor with `@`-prefix file attachment fuzzy search and `/`-prefix command mode, a loading spinner while the model is generating, and popups for file/command suggestions.

The widget is part of `ratkit::widgets::ai_chat` and requires the `ai-chat` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
ai-chat = ["ratkit/ai-chat"]
```

Without this flag, `AIChat` and all related types are not compiled.

---

## 2. Construct the Widget

`AIChat::new()` returns a fully configured widget with sensible defaults: a cyan bold style for user messages, white for AI messages, a `You: ` prompt, and a single built-in `/clear` command. No sub-states are required.

```rust
use ratkit::widgets::ai_chat::{AIChat, InputState, Message, MessageRole, MessageStore};

let mut chat = AIChat::new();

// Optional: register custom commands
chat.register_command("/help".to_string());
chat.register_command("/compact".to_string());

// Optional: pre-seed message history
chat.messages_mut().add(Message::user("Hi!".to_string()));
chat.messages_mut().add(Message::assistant("Hello!".to_string()));
```

The widget owns its `MessageStore` and `InputState` internally. Use the `messages_mut()` and `input_mut()` accessors to push state in from the host application.

---

## 3. Configure via Builder Methods

Chain configuration methods on the widget handle **before** entering the event loop. These affect every subsequent render and key handling call.

| Method | Effect |
|--------|--------|
| `.with_user_message_style(style)` | Style applied to user-role messages |
| `.with_ai_message_style(style)` | Style applied to assistant-role messages |
| `.with_input_style(style)` | Style for the input editor text |
| `.with_prompt(prompt)` | Prompt prefix shown before user input (default `"You: "`) |
| `.with_selected_command_index(idx)` | Initial command selection in `/`-mode |
| `.register_command(cmd)` | Adds a slash command to the palette |
| `.set_loading(bool)` | Toggles the AI-generating spinner |

`Message::user(...)` and `Message::assistant(...)` accept plain strings. Use `.with_attachment(path)` to tag a file path on a user message.

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. Your application struct must implement `CoordinatorApp`, which defines two methods:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events and must route them to the widget:

**Keyboard events:**
1. Extract the `crossterm::event::KeyCode` from `CoordinatorEvent::Keyboard(keyboard)`.
2. Pass it to `chat.handle_key(key_code)`.
3. Match the returned `AIChatEvent` to handle submission, file selection, or command dispatch.
4. Return `CoordinatorAction::Redraw` when state changes; `Continue` otherwise.

**Mouse events:**
- The widget itself does not consume mouse events. Forward them only to your outer layout.

**Other events:**
- `CoordinatorEvent::Tick` — call `chat.set_loading(false)` when the model finishes, or drive periodic UI updates.
- `CoordinatorEvent::Resize` — always return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Compute your layout area with `ratatui::Layout`.
2. Call `chat.render(frame, area)`. The widget handles its own internal splitting (messages over input) and popup overlays.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(3)])
    .split(frame.area());

let chat_area = chunks[0]; // body, before status line
```

Reserve a `Min(0)` region for `chat.render(...)`. The widget draws its own input row at the bottom of the area it owns.

---

## 6. Handle `AIChatEvent` Variants

`chat.handle_key()` returns an `AIChatEvent` enum. Match on its variants to trigger application-level behavior:

| Variant | Meaning | Required Action |
|---------|---------|-----------------|
| `AIChatEvent::None` | No state change | Return `Continue` |
| `AIChatEvent::MessageSubmitted(text)` | User pressed Enter on plain text | Stream the prompt to your model |
| `AIChatEvent::FileAttached(path)` | User confirmed an `@`-file | Embed the file in the next prompt |
| `AIChatEvent::Command(name)` | User confirmed a `/`-command | Dispatch locally (e.g. `/clear` clears the store) |

`AIChat` already inserts `Message::user(...)` into the store on submission. You only need to act on the event and then call `chat.set_loading(true)` while waiting for the model.

---

## 7. Coalesce Mouse-Move Events

Terminal emulators emit mouse-move events at 50–200 Hz. The `AIChat` widget is keyboard-driven, but if you layer other widgets that react to hover, coalesce move events at the host level:

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

Non-motion events (click, scroll) should always pass through.

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
| 1 | Enable `ai-chat` feature flag | Widget not compiled, build fails |
| 2 | Construct with `AIChat::new()` and optional builder chain | Defaults applied, no failures |
| 3 | Pre-seed `messages_mut()` if restoring a session | Conversation starts empty |
| 4 | Store layout `Rect` from `Layout::split()` in app state | Widget renders into wrong area |
| 5 | Route `handle_key()` with `crossterm::event::KeyCode` | Keyboard input ignored |
| 6 | Match `AIChatEvent` variants from `handle_key()` | User prompts never reach the model |
| 7 | Toggle `set_loading(true/false)` around model calls | No spinner, no UX feedback |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Render with `chat.render(frame, area)` | Nothing renders on screen |
| 10 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::ai_chat`:

- `AIChat` — The main widget struct
- `AIChatEvent` — Enum of events emitted by `handle_key()`: `None`, `MessageSubmitted(String)`, `FileAttached(String)`, `Command(String)`
- `Message` — Single chat message with `role`, `content`, `attachments`, and `timestamp`
- `MessageRole` — Enum: `User`, `Assistant`
- `MessageStore` — Conversation history (`add`, `clear`, `messages`, `last`, `len`, `is_empty`)
- `InputState` — Multi-line text input with `@`-file and `/`-command prefix parsing
