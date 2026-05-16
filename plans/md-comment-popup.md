# Markdown Comment Popup Plan

## Goal

Add line comments to the markdown widget without making the widget responsible for persistence storage decisions.

Users can focus/select a markdown line, press a customizable hotkey, type a multiline comment in a popup shown below that line, and submit it. Lines with comments show a circle marker in the right gutter.

## Confirmed Decisions

1. **Persistence owner**
   - The host app owns persistence.
   - The widget receives comment data from the host.
   - The widget emits typed events when comments are submitted.
   - The host decides where and how comments are stored.

2. **Storage location metadata**
   - The widget can be passed metadata describing where comments are stored.
   - The widget must not derive, infer, or manage the storage location by itself.
   - This metadata is forwarded back in events when needed so the host can route persistence correctly.

3. **Comment anchoring**
   - Comments are anchored by line number plus line content.
   - The stored target should include:
     - 1-indexed line number.
     - A stable hash or snapshot of the line content.
     - Optional visible line snippet for debugging/display.
   - If the line is removed or the content no longer matches, the old comment should not be shown on the new unrelated line.

4. **Popup behavior**
   - The popup supports multiline text.
   - Enter inserts a newline.
   - Ctrl+Enter submits the comment and closes the popup.
   - Esc cancels the popup without submitting.

5. **Event contract**
   - Submitting emits a typed markdown event.
   - Preferred event shape:

```rust
MarkdownEvent::CommentSubmitted {
    line: usize,
    line_hash: String,
    line_text: String,
    comment_text: String,
    storage_ref: Option<String>,
}
```

## Widget API Requirements

### Configurable hotkey

The widget should expose a configurable hotkey for toggling the comment popup.

Possible config shape:

```rust
pub struct CommentPopupConfig {
    pub toggle_hotkey: CommentHotkey,
    pub storage_ref: Option<String>,
}
```

The default hotkey can be chosen later, but it must be overridable by the host.

### Comments input

The host should pass comments into the widget as data, not as a storage adapter.

Possible data shape:

```rust
pub struct MarkdownLineComment {
    pub line: usize,
    pub line_hash: String,
    pub line_text: String,
    pub comment_count: usize,
}
```

The widget uses this only for rendering markers and validating whether a comment still applies to the current line.

### Event output

The widget emits a typed submit event when Ctrl+Enter is pressed inside the popup.

The host receives this event and persists the comment.

## Rendering Requirements

1. When the focused line has comments, render a circle marker in the right gutter.
2. The marker must only appear when the current line content still matches the stored line hash/content anchor.
3. When the popup is open, render it below the focused line where possible.
4. If there is not enough vertical space below the focused line, render the popup above or clamp it within the markdown widget viewport.
5. The popup should visually separate itself from markdown content with a border/title.

## Input Requirements

1. Normal mode:
   - Pressing the configured hotkey opens the popup for the focused line.
   - If the popup is already open for that line, pressing the hotkey closes it.
   - If the popup is open for another line, pressing the hotkey moves it to the current focused line.

2. Popup mode:
   - Regular characters are inserted into the popup buffer.
   - Enter inserts a newline.
   - Backspace deletes text.
   - Esc cancels and closes the popup.
   - Ctrl+Enter emits `MarkdownEvent::CommentSubmitted` and closes the popup.

## State Requirements

The widget should keep transient popup state only:

```rust
pub struct CommentPopupState {
    pub active: bool,
    pub line: usize,
    pub line_hash: String,
    pub line_text: String,
    pub buffer: String,
}
```

It should not keep authoritative persisted comments internally.

## Non-goals

1. The widget must not choose a file path or database location for comments.
2. The widget must not read or write comment persistence directly.
3. The widget must not show comments for lines whose content anchor no longer matches.
4. The first implementation does not need threaded comment browsing unless added later.

## Open Questions

1. Exact default hotkey.
2. Exact right-gutter circle glyph.
3. Whether comment markers should show count, hover state, or just existence.
4. Whether the popup should allow editing existing draft text if toggled closed and reopened before submit.
