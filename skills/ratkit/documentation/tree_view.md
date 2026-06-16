# TreeView Primitive Integration Guide

## Overview

`TreeView` is a generic hierarchical tree-view primitive integrated into `ratkit`. It is a `StatefulWidget` parameterized over the data type `T` stored in each `TreeNode<T>`. It supports selection, expansion, scrolling, an optional inline filter UI, custom node rendering via a closure, keyboard navigation through `TreeNavigator`, and mouse hit-testing.

The primitive is part of `ratkit::primitives::tree_view` and requires the `tree-view` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The primitive is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
tree-view = ["ratkit/tree-view"]
```

Without this flag, `TreeView`, `TreeNode`, and friends are not compiled.

---

## 2. Build the Node Tree

`TreeNode<T>` is generic over the data type. Use `TreeNode::new(data)` for a leaf and `TreeNode::with_children(data, vec![...])` for a parent:

```rust
use ratkit::primitives::tree_view::{TreeNode, TreeView, TreeViewState, TreeNavigator};

let nodes: Vec<TreeNode<&'static str>> = vec![
    TreeNode::with_children("src", vec![
        TreeNode::new("lib.rs"),
        TreeNode::new("main.rs"),
        TreeNode::with_children("widgets", vec![
            TreeNode::new("button.rs"),
            TreeNode::new("dialog.rs"),
        ]),
    ]),
    TreeNode::with_children("tests", vec![
        TreeNode::new("smoke.rs"),
    ]),
];

let mut state = TreeViewState::new();
let navigator = TreeNavigator::new();
```

Public fields on `TreeNode<T>`: `data: T`, `children: Vec<TreeNode<T>>`, `expandable: bool`. Public fields on `TreeViewState`: `selected_path: Option<Vec<usize>>`, `expanded: HashSet<Vec<usize>>`, `offset: usize`, `filter: Option<String>`, `filter_mode: bool`.

`TreeNode::new(data)` creates a non-expandable leaf. `TreeNode::with_children(data, vec![...])` creates an expandable parent. The `expandable` flag is set automatically from whether children are present, but the type stores it explicitly so the renderer can branch without walking the children list.

---

## 3. Configure via Builder Methods

`TreeView::new(nodes)` returns a tree with the default triangle expand/collapse icons (`▶` and `▾`) and a placeholder `render_fn` that prints `"Node"`. Chain the per-frame setters:

```rust
let tree = TreeView::new(nodes)
    .render_fn(|data, state| {
        if state.is_selected {
            Line::from(format!("> {}", data))
        } else {
            Line::from(*data)
        }
    })
    .highlight_style(Style::default().bg(Color::DarkGray))
    .block(Block::default().borders(Borders::ALL).title(" Tree "))
    .with_filter_ui(true);
```

| Method | Effect |
|--------|--------|
| `.render_fn(\|&T, &NodeState| -> Line<'a>)` | Per-node render closure |
| `.highlight_style(Style)` | Full-width background style for the selected row |
| `.block(Block<'a>)` | Wrap the tree in a bordered block |
| `.with_filter_ui(bool)` | Show the inline `/`-filter prompt at the bottom |
| `.icons(expand: &'a str, collapse: &'a str)` | Replace the default triangles |
| `.handle_key_event(KeyEvent, &TreeNavigator, &mut TreeViewState) -> WidgetEvent` | Free-style keyboard routing |
| `.handle_mouse_event(MouseEvent, &mut TreeViewState, Rect) -> WidgetEvent` | Mouse click and scroll routing |

`NodeRenderFn<'a, T> = Box<dyn Fn(&T, &NodeState) -> Line<'a> + 'a>` is the closure type. `NodeState` carries `depth`, `is_expanded`, `is_selected`, `has_children`, and `path` fields.

For borrowing data without cloning, use `TreeViewRef<'a, T>` (configured the same way but takes `&'a [TreeNode<T>]`).

---

## 4. Integrate with `CoordinatorApp`

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

**Keyboard events:**
1. Extract `KeyEvent` from `CoordinatorEvent::Keyboard(keyboard)`.
2. If the user pressed `/` and `with_filter_ui(true)`, call `state.enter_filter_mode()`.
3. Otherwise build the tree and call `tree.handle_key_event(key, &navigator, &mut state)`.
4. Return `Redraw` on any state change.

**Mouse events:**
- Call `tree.handle_mouse_event(mouse, &mut state, area)` with the area you render into. The function sets `state.selected_path` and returns `WidgetEvent::Selected { path }` on click.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue`.
- `CoordinatorEvent::Resize` — return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

```rust
use ratatui::layout::Rect;
use ratatui::Frame;

let area: Rect = frame.area();
let tree = self.build_tree();
frame.render_stateful_widget(tree, area, &mut self.state);
```

`build_tree()` is a helper on your app struct that rebuilds the `TreeView` each frame with the latest `render_fn`, `highlight_style`, and `block`. The widget takes ownership of the tree value but the state lives across frames.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`. A common setup:

```rust
use ratatui::layout::{Constraint, Direction, Layout};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(1)])
    .split(frame.area());
```

If `with_filter_ui(true)`, the tree reserves the last row of the area for the `/` filter prompt.

---

## 6. Handle `WidgetEvent` Variants

Both `handle_key_event` and `handle_mouse_event` return `WidgetEvent`. The variants relevant to the tree are:

| Variant | Meaning | Required Action |
|---------|---------|-----------------|
| `WidgetEvent::None` | No state change | Return `Continue` |
| `WidgetEvent::Selected { path }` | User clicked or pressed Enter | Dispatch the selection to your app |
| `WidgetEvent::FilterModeExited { path }` | User exited filter mode (Enter or Esc) | Trigger `Redraw` |
| `WidgetEvent::FilterModeChanged { active, filter }` | User typed into the `/` filter | Trigger `Redraw` |

The full `WidgetEvent` enum is shared by all primitives that consume the type (see the `widget_event` feature module for the complete list).

---

## 7. Navigation Helpers

`TreeNavigator` owns the keybindings. Construct it with `TreeNavigator::new()` for the defaults or use `TreeKeyBindings` to customize:

```rust
use ratkit::primitives::tree_view::TreeNavigator;

let navigator = TreeNavigator::new();
```

The default keys are `j`/`Down` to move down, `k`/`Up` to move up, `h`/`Left` to collapse, `l`/`Right` to expand, `Enter` to toggle, `g` to go to top, `G` to go to bottom, and `/` to enter filter mode (when `with_filter_ui(true)`).

Helper functions on `TreeView` (also re-exported from the module root):

| Function | Purpose |
|----------|---------|
| `get_visible_paths(&TreeViewState) -> Vec<Vec<usize>>` | All visible node paths in display order |
| `get_visible_paths_filtered(&TreeViewState, filter) -> Vec<Vec<usize>>` | Same, but filtered by a substring match |
| `matches_filter(data: &T, filter: &str) -> bool` | Default filter predicate (string contains) |

---

## 8. Coalesce Mouse-Move Events

The tree does not consume mouse-move events. If other widgets in the same app react to hover, coalesce at the host level:

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

## 9. Run the Application

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
| 1 | Enable `tree-view` feature flag | Primitive not compiled, build fails |
| 2 | Build `Vec<TreeNode<T>>` with `TreeNode::new` and `TreeNode::with_children` | Tree has no rows |
| 3 | Construct `TreeView::new(nodes)` and chain `.render_fn(...)` and `.block(...)` | Default placeholder text, no border |
| 4 | Pair with `TreeViewState::new()` and (optionally) `TreeNavigator::new()` | State lost between frames |
| 5 | Rebuild the tree every frame and call `handle_key_event` / `handle_mouse_event` | Keyboard/mouse ignored |
| 6 | Match `WidgetEvent` variants to react to selection / toggle / scroll | Selection silently dropped |
| 7 | Render with `frame.render_stateful_widget(tree, area, &mut state)` | Nothing renders on screen |
| 8 | Coalesce mouse-move events (< 24ms) | UI freezes from queue saturation |
| 9 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::primitives::tree_view`:

- `TreeNode<T>` — `data: T`, `children: Vec<TreeNode<T>>`, `expandable: bool`
- `NodeState` — `depth`, `is_expanded`, `is_selected`, `has_children`, `path`
- `NodeRenderFn<'a, T>`, `NodeFilterFn`, `NodeRenderRefFn`
- `TreeView<'a, T>` — Owns its nodes; `StatefulWidget` over `TreeViewState`
- `TreeViewRef<'a, T>` — Borrows its nodes; same configuration API
- `TreeViewState` — `selected_path`, `expanded`, `offset`, `filter`, `filter_mode`
- `TreeNavigator`, `TreeKeyBindings`
- `get_visible_paths`, `get_visible_paths_filtered`, `matches_filter`

Key methods on `TreeView`:

- `TreeView::new(nodes)`, `.render_fn(...)`, `.highlight_style(...)`, `.block(...)`, `.with_filter_ui(...)`, `.expand_icon(...)`, `.collapse_icon(...)`
- `handle_key_event(KeyEvent, &TreeNavigator, &mut TreeViewState) -> WidgetEvent`
- `handle_mouse_event(MouseEvent, &mut TreeViewState, Rect) -> WidgetEvent`
- `flatten_tree(&TreeViewState)`, `node_at_row(usize, &TreeViewState)`, `visible_item_count(&TreeViewState)`
