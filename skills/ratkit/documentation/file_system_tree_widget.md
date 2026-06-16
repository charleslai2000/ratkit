# FileSystemTreeWidget Integration Guide

## Overview

`FileSystemTree` is a tree widget integrated into `ratkit` for browsing a real on-disk directory. It loads entries lazily as directories expand, supports vim-style keyboard navigation, a `/`-prefix filter mode, devicon-style file icons, and a `Block` wrapper for borders and titles. The widget is a `StatefulWidget` over `FileSystemTreeState`.

The widget is part of `ratkit::widgets::file_system_tree` and requires the `file-system-tree` feature flag to be enabled at compile time.

---

## 1. Feature Flag

The widget is behind a feature gate. Your `Cargo.toml` must enable it:

```toml
[features]
file-system-tree = ["ratkit/file-system-tree"]
```

Without this flag, `FileSystemTree` and all related types are not compiled.

---

## 2. Construct the Widget and State

`FileSystemTree::new(root_path)` reads the root directory and returns a widget with one root node. Pair it with a `FileSystemTreeState`:

```rust
use std::path::PathBuf;
use ratkit::widgets::file_system_tree::{FileSystemTree, FileSystemTreeState};

let tree = FileSystemTree::new(PathBuf::from("."))?;
let mut state = FileSystemTreeState::new();
state.select(vec![0]); // pre-select the root entry
```

`FileSystemTree::new(...)` returns `io::Result<Self>`. It loads only the root's *direct* children — child directories are read on demand by `expand_directory(...)` when the user expands them. This keeps the initial load fast for large trees.

The state holds:

| Field | Purpose |
|-------|---------|
| `selected_path: Option<Vec<usize>>` | Indices from root to the currently selected node |
| `expanded: HashSet<Vec<usize>>` | Set of paths that are currently expanded |
| `offset: usize` | First visible row for scrolling |
| `filter: Option<String>` | Active filter string |
| `filter_mode: bool` | Whether the `/` prompt is currently being edited |

---

## 3. Configure via Builder Methods

| Method | Effect |
|--------|--------|
| `.with_config(root_path, FileSystemTreeConfig) -> io::Result<Self>` | Replace icon and style configuration (re-reads the root) |
| `.block(Block<'a>)` | Wrap the tree in a bordered block with a title |

`FileSystemTreeConfig` controls selection/dir/file styles, the dark/light devicon theme, and ignore settings:

| Field | Default | Purpose |
|-------|---------|---------|
| `selected_style: Style` | white on blue | Highlight style for the selected row |
| `dir_style: Style` | blue | Style for unselected directories |
| `file_style: Style` | white | Style for unselected files |
| `use_dark_theme: bool` | `true` | Switches `devicons::Theme` between `Dark` and `Light` |

---

## 4. Integrate with `CoordinatorApp`

`ratkit` uses a coordinator pattern. Your application struct must implement `CoordinatorApp`, which defines two methods:

### `on_event(&mut self, event: CoordinatorEvent) -> LayoutResult<CoordinatorAction>`

This method receives terminal events and must route them to the widget:

**Keyboard events:**
1. Extract `KeyCode` from `CoordinatorEvent::Keyboard(keyboard)`.
2. If `tree.is_filter_mode(&state)`, route to `tree.handle_filter_key(key, &mut state)`.
3. Otherwise, route to `tree.handle_navigation_key(key, &mut state)`. Built-in keys: `j`/`Down`, `k`/`Up`, `h`/`Left`, `l`/`Right`, `Enter`.
4. The `/` key enters filter mode (via `tree.enter_filter_mode(&mut state)`).
5. Return `CoordinatorAction::Redraw` when state changes; `Continue` otherwise.

**Mouse events:**
- The widget does not consume mouse events directly. If you wire them, walk `tree.get_visible_paths(&state)` and match on `tree.get_entry_at_path(&path)`.

**Other events:**
- `CoordinatorEvent::Tick` — return `Continue` unless refreshing disk contents.
- `CoordinatorEvent::Resize` — always return `Redraw`.

### `on_draw(&mut self, frame: &mut Frame)`

1. Define your layout with `ratatui::Layout`.
2. Call `frame.render_stateful_widget(self.tree.clone().block(...), area, &mut self.state)`. The widget handles its own internal rendering, including the filter line at the bottom.

---

## 5. Layout Is Pure `ratatui`

`ratkit` does **not** provide its own layout engine. The layout system is 100% `ratatui`.

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([Constraint::Min(0), Constraint::Length(3)])
    .split(frame.area());

self.tree_area = chunks[0];
```

The widget reserves the **last row** of the area for the filter prompt when filter mode is active. Allocate at least 2 rows for the tree if you want filter mode to be visible.

---

## 6. Built-in Navigation Reference

`tree.handle_navigation_key(...)` covers the standard subset:

| Key | Action |
|-----|--------|
| `j` / `Down` | Select next visible entry |
| `k` / `Up` | Select previous visible entry |
| `l` / `Right` | Expand selected directory, or move to first child |
| `h` / `Left` | Collapse selected directory, or move to parent |
| `Enter` | Toggle expansion of the selected directory |

`tree.handle_filter_key(...)` covers filter-mode input:

| Key | Action |
|-----|--------|
| Any char | Append to filter |
| `Backspace` | Pop one character from filter |
| `Enter` | Confirm filter and exit filter mode |
| `Esc` | Cancel filter and exit filter mode |

---

## 7. Persist and Refresh

The state is plain data and trivially cloneable. To re-scan the disk (e.g. on `Tick` or after a file-system watcher fires), drop the cached `nodes` and rebuild the widget:

```rust
use std::path::PathBuf;
use ratkit::widgets::file_system_tree::FileSystemTree;

if let Ok(new_tree) = FileSystemTree::new(PathBuf::from(".")) {
    self.tree = new_tree;
}
```

`expand_directory(&[usize])` is also public — call it manually for hosts that want to pre-expand specific paths.

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
| 1 | Enable `file-system-tree` feature flag | Widget not compiled, build fails |
| 2 | Construct `FileSystemTree::new(root)` and `FileSystemTreeState::new()` | Widget panics or shows nothing |
| 3 | Chain `.with_config(...)` / `.block(...)` if you want a title | Default style + no border |
| 4 | Store layout `Rect` from `Layout::split()` in app state | Widget renders into wrong area |
| 5 | Route `handle_navigation_key()` for vim keys and arrows | Keyboard input ignored |
| 6 | Route `handle_filter_key()` after entering filter mode with `/` | Filter prompt does not work |
| 7 | Render with `frame.render_stateful_widget(tree, area, &mut state)` | Nothing renders on screen |
| 8 | Call `run(app, config)` to start the event loop | No TUI loop runs |

---

## Type Reference

All types live under `ratkit::widgets::file_system_tree`:

- `FileSystemTree<'a>` — `StatefulWidget` over `FileSystemTreeState`
- `FileSystemTreeState` — `selected_path`, `expanded`, `offset`, `filter`, `filter_mode`
- `FileSystemTreeConfig` — `selected_style`, `dir_style`, `file_style`, `use_dark_theme`
- `FileSystemEntry` — One disk entry: `name`, `path`, `is_dir`
- `FileSystemTreeNode` — Tree node wrapping `FileSystemEntry` with `children: Vec<...>` and `expandable: bool`

Key methods on `FileSystemTree`:

- `new(root_path) -> io::Result<Self>` — read the root
- `with_config(root_path: PathBuf, FileSystemTreeConfig) -> io::Result<Self>` — replace config (re-reads root)
- `block(Block<'a>) -> Self` — wrap with a bordered block
- `expand_directory(&[usize]) -> io::Result<()>` — load children on demand
- `get_entry_at_path(&[usize]) -> Option<&FileSystemEntry>` — resolve a path
- `get_selected_entry(&state) -> Option<&FileSystemEntry>` — current selection
- `get_visible_paths(&state) -> Vec<Vec<usize>>` — flattened visible paths
- `select_next(&mut state)`, `select_previous(&mut state)`, `toggle_selected(&mut state)`, `expand_selected(&mut state)`, `collapse_selected(&mut state)`
- `handle_navigation_key(KeyCode, &mut state) -> io::Result<bool>`
- `enter_filter_mode(&mut state)`, `is_filter_mode(&state)`, `filter_text(&state)`, `clear_filter(&mut state)`
- `handle_filter_key(KeyCode, &mut state) -> bool`
