# Code Diff Upgrade Plan

## Goal

Upgrade `ratkit`'s `code-diff` widget into a reusable, library-grade diff review component inspired by Lumen's strongest diff UX patterns, without copying Lumen as an application or embedding app-specific GitHub/AI behavior in the widget.

The result should be a composable ratatui widget that supports side-by-side code review, file navigation, hunk navigation, word-level highlighting, optional syntax highlighting, and host-owned review state.

## Evidence Reviewed

1. Lumen crate page on lib.rs:
   - Lumen is a Rust command-line utility/TUI for AI-assisted git workflows and code review.
   - It advertises side-by-side diff review, GitHub PR review, annotations, watch mode, stacked commits, and AI helpers.
2. Lumen README:
   - The diff viewer supports uncommitted changes, commits, branch ranges, PR URLs, file filters, watch mode, stacked mode, focus-on-file, themes, selections, annotations, hunk navigation, and viewed-file tracking.
3. Lumen source layout:
   - Diff concerns are split across `types`, `state`, `diff_algo`, `render`, `highlight`, `theme`, `watcher`, and VCS adapters.
4. Current `ratkit` source:
   - `src/widgets/code_diff` currently has lightweight unified-diff parsing and a placeholder widget render.
   - Supporting modules already exist for `file-system-tree`, `theme-picker`, `git-watcher`, and `file-watcher`.

## Scope

1. Improve the reusable `code-diff` widget.
2. Add pure diff parsing and side-by-side layout logic.
3. Add optional host-integrated file tree support.
4. Add host-owned review state contracts for viewed files, focus, selections, and annotations.
5. Add optional syntax highlighting behind feature flags.
6. Add examples and tests proving behavior.

## Non-goals

1. Do not port Lumen's full CLI.
2. Do not add natural-language git command execution.
3. Do not execute shell commands from the widget.
4. Do not make `code-diff` fetch GitHub PRs directly.
5. Do not make the widget own persistence for comments, annotations, or viewed files.
6. Do not require tree-sitter for the default `code-diff` feature.
7. Do not create files above 200 lines; split every single-purpose function into module files.

## Design Principles

1. **Widget purity**
   - `code-diff` renders and emits typed events.
   - Host applications own VCS IO, persistence, network access, and AI workflows.

2. **Small modules**
   - Every function with one job goes in its own file under a domain folder.
   - Every public function and type must have jsdoc-style Rust doc comments.

3. **Feature-gated complexity**
   - Keep baseline diff rendering dependency-light.
   - Add syntax highlighting and repository adapters as optional features.

4. **Host-owned state**
   - The widget accepts state snapshots and emits events.
   - The host decides how to persist, reload, or sync state.

5. **TDD delivery**
   - Each behavior begins with a failing test.
   - Implement the minimum passing code.
   - Refactor only after green tests.

## Proposed Module Layout

```text
src/widgets/code_diff/
  mod.rs
  code_diff/
    mod.rs
    state/
      mod.rs
      diff_view_state.rs
      file_review_state.rs
      selection_state.rs
      hunk_focus_state.rs
    types/
      mod.rs
      change_type.rs
      diff_file.rs
      diff_hunk.rs
      diff_line.rs
      inline_segment.rs
      side_by_side_row.rs
    parse/
      mod.rs
      parse_unified_diff.rs
      parse_hunk_header.rs
      parse_file_header.rs
    layout/
      mod.rs
      build_side_by_side_rows.rs
      pair_modified_lines.rs
      compute_inline_segments.rs
      expand_tabs.rs
    render/
      mod.rs
      render_code_diff.rs
      render_diff_header.rs
      render_line_gutter.rs
      render_side_by_side_row.rs
      render_unified_row.rs
      render_empty_state.rs
    events/
      mod.rs
      code_diff_event.rs
      handle_code_diff_key.rs
      handle_code_diff_mouse.rs
    theme/
      mod.rs
      diff_theme.rs
      default_diff_theme.rs
    highlight/
      mod.rs
      highlight_provider.rs
      no_highlight_provider.rs
      syntect_highlight_provider.rs
      tree_sitter_highlight_provider.rs
    annotations/
      mod.rs
      annotation_anchor.rs
      annotation_marker.rs
      annotation_event.rs
    file_tree/
      mod.rs
      diff_file_tree_adapter.rs
```

## Feature Flags

### Existing

`code-diff` should remain available as a small default-capable widget feature using the existing `similar` dependency.

### New optional features

1. `code-diff-syntect`
   - Enables syntax highlighting using the existing `syntect` dependency path.
   - Depends on `code-diff`, `syntect`, and `syntect-tui`.

2. `code-diff-tree-sitter`
   - Enables tree-sitter highlighting for selected languages.
   - Should be optional because it adds many dependencies.

3. `code-diff-file-tree`
   - Enables adapters between `code-diff` and the existing `file-system-tree` widget.

4. `code-diff-review`
   - Enables selection, viewed-state markers, and annotation event contracts.
   - Does not enable persistence.

## Data Model

### Diff file

```rust
pub struct DiffFile {
    pub path: String,
    pub old_path: Option<String>,
    pub status: DiffFileStatus,
    pub hunks: Vec<DiffHunk>,
}
```

### Side-by-side row

```rust
pub struct SideBySideRow {
    pub old_line: Option<DiffLineCell>,
    pub new_line: Option<DiffLineCell>,
    pub change_type: ChangeType,
}
```

### Inline segment

```rust
pub struct InlineSegment {
    pub text: String,
    pub emphasized: bool,
}
```

### Widget state

```rust
pub struct CodeDiffState {
    pub selected_file_index: usize,
    pub scroll_offset: usize,
    pub focused_hunk_index: Option<usize>,
    pub focused_panel: DiffPanel,
    pub selection: Option<DiffSelection>,
    pub viewed_files: std::collections::HashSet<String>,
}
```

The state is mutable and host-owned, following ratatui widget conventions.

## Event Contract

The widget should emit typed events instead of performing side effects.

```rust
pub enum CodeDiffEvent {
    FileSelected { path: String },
    FileViewedToggled { path: String, viewed: bool },
    HunkFocused { path: String, hunk_index: usize },
    SelectionChanged { selection: DiffSelection },
    SelectionCopied { text: String },
    AnnotationRequested { target: AnnotationTarget },
    OpenInEditorRequested { path: String, line: Option<usize> },
}
```

The host decides whether file viewed state syncs to GitHub, a local database, memory, or nowhere.

## Implementation Phases

### Phase 1: Pure parsing foundation

1. Add parser tests for unified diffs with:
   - one file,
   - multiple files,
   - added files,
   - deleted files,
   - renamed files,
   - binary file markers,
   - hunk headers with and without explicit counts.
2. Implement `parse_unified_diff` returning `Vec<DiffFile>`.
3. Replace the current placeholder parser in `CodeDiff::from_unified_diff` with the new parser.
4. Keep public API backward compatible where possible.

Done when parser tests pass and the old `CodeDiff::from_unified_diff` behavior still works for simple diffs.

### Phase 2: Side-by-side row builder

1. Add tests for pairing consecutive deletes and inserts into modified rows.
2. Add tests for pure inserts, pure deletes, and unchanged context lines.
3. Implement `build_side_by_side_rows`.
4. Implement `pair_modified_lines` separately.
5. Implement `compute_inline_segments` using `similar::TextDiff::diff_unicode_words`.
6. Only show inline emphasis when unchanged content ratio is above a documented threshold.

Done when modified lines render as aligned old/new rows instead of visually offset delete-then-insert blocks.

### Phase 3: Real rendering

1. Replace placeholder `Widget for CodeDiff` render output with actual diff rows.
2. Add configurable modes:
   - `DiffStyle::SideBySide`,
   - `DiffStyle::Unified`.
3. Render:
   - file header,
   - line-number gutters,
   - old/new panes,
   - inserted/deleted/modified/context row backgrounds,
   - hunk headers,
   - empty placeholder area.
4. Add snapshot tests using `ratatui::backend::TestBackend`.

Done when the widget renders readable diff content in both side-by-side and unified modes.

### Phase 4: Navigation state

1. Add `CodeDiffState` and stateful rendering APIs.
2. Add hunk navigation:
   - next hunk,
   - previous hunk,
   - scroll-to-focused-hunk.
3. Add file navigation:
   - next file,
   - previous file,
   - selected file by index.
4. Emit typed events on navigation changes.

Done when examples can move between files and hunks without app-specific logic inside the widget.

### Phase 5: File tree integration

1. Build an adapter from `Vec<DiffFile>` into the existing `file-system-tree` structure.
2. Show status indicators for added, modified, deleted, renamed, and viewed files.
3. Keep this under `code-diff-file-tree`.
4. Do not make the diff widget require file-tree rendering.

Done when a host app can compose `FileSystemTree` beside `CodeDiff` with shared selected-file state.

### Phase 6: Selection and annotations contract

1. Add line-level selection state.
2. Add optional character-level selection only after line selection is stable.
3. Add annotation target types:
   - file,
   - hunk,
   - line range,
   - selection.
4. Add marker rendering from host-provided annotation metadata.
5. Emit `AnnotationRequested` events; do not persist annotations.

Done when the widget supports review workflows while keeping storage outside ratkit.

### Phase 7: Syntax highlighting

1. Define a `HighlightProvider` trait.
2. Add `NoHighlightProvider` as the default.
3. Add `SyntectHighlightProvider` behind `code-diff-syntect`.
4. Add `TreeSitterHighlightProvider` behind `code-diff-tree-sitter` only after dependency review.
5. Cache highlights per file and line to avoid re-highlighting every frame.

Done when highlighting can be enabled without changing core rendering APIs.

### Phase 8: Watcher and VCS adapters outside widget core

1. Keep repository fetching in services, not the widget.
2. Add optional helper examples that combine:
   - `git-watcher`,
   - `file-watcher`,
   - host-loaded diff text,
   - `CodeDiff` state refresh.
3. If VCS abstraction is added, place it under services, not `widgets/code_diff`.
4. Do not add GitHub API calls to `code-diff`.

Done when watch-mode-style UX can be built by hosts without coupling network or git logic to the widget.

### Phase 9: Examples and docs

1. Update the existing `code_diff_code_diff_demo` example.
2. Add a side-by-side demo with static diff fixtures.
3. Add a file-tree + diff demo behind `code-diff-file-tree`.
4. Document:
   - state ownership,
   - event flow,
   - feature flags,
   - host integration patterns.

Done when users can copy an example to embed `CodeDiff` in their own app.

## Testing Plan

1. Unit tests:
   - `parse_hunk_header`,
   - `parse_unified_diff`,
   - `build_side_by_side_rows`,
   - `pair_modified_lines`,
   - `compute_inline_segments`,
   - `expand_tabs`.
2. Snapshot tests:
   - unified rendering,
   - side-by-side rendering,
   - long-line truncation,
   - selected file state,
   - hunk focus state.
3. Interaction tests:
   - key handling for hunk navigation,
   - file selection,
   - viewed toggle event,
   - annotation request event.
4. Feature tests:
   - default build without highlighting,
   - `code-diff-syntect`,
   - `code-diff-file-tree`.

## Risks

1. Tree-sitter dependencies can make builds heavy.
   - Mitigation: keep tree-sitter optional and start with `syntect` or no highlighting.
2. Rendering can become a god-file.
   - Mitigation: split every render operation into focused files under `render/`.
3. Widget may accidentally become an app.
   - Mitigation: typed events only; no persistence, shell execution, GitHub sync, or AI requests inside the widget.
4. Diff parsing edge cases are broad.
   - Mitigation: fixture-driven tests before expanding behavior.

## Recommended First PR

Implement only Phase 1 and Phase 2:

1. Add new parser and layout modules.
2. Add tests for unified parsing and side-by-side row generation.
3. Keep rendering changes minimal.
4. Do not add tree-sitter, file tree, annotations, or VCS services yet.

This creates the foundation for better UX while staying small, testable, and reversible.

## Acceptance Criteria

1. `cargo test --features code-diff` passes.
2. `CodeDiff::from_unified_diff` parses multi-file diffs.
3. Side-by-side rows align modified lines on the same row.
4. Inline word segments identify changed words for sufficiently similar modified lines.
5. No new file exceeds 200 lines.
6. Every public function and type has Rust doc comments.
7. No widget code performs filesystem persistence, shell execution, network requests, or AI calls.
