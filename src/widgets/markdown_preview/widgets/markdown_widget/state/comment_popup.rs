use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Configures the key combination that toggles the markdown comment popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentHotkey {
    /// Key code that activates the hotkey.
    pub code: KeyCode,
    /// Required modifiers for the hotkey.
    pub modifiers: KeyModifiers,
}

impl CommentHotkey {
    /// Creates a new comment hotkey from a crossterm key code and modifiers.
    pub fn new(code: KeyCode, modifiers: KeyModifiers) -> Self {
        Self { code, modifiers }
    }

    /// Returns true when the provided key event matches this hotkey.
    pub fn matches(&self, key: &KeyEvent) -> bool {
        key.code == self.code && key.modifiers == self.modifiers
    }
}

impl Default for CommentHotkey {
    /// Creates the default comment popup hotkey.
    fn default() -> Self {
        Self {
            code: KeyCode::Char('c'),
            modifiers: KeyModifiers::NONE,
        }
    }
}

/// Configures comment popup behavior supplied by the host application.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommentPopupConfig {
    /// Hotkey that opens, closes, or retargets the comment popup.
    pub toggle_hotkey: CommentHotkey,
    /// Opaque host-owned storage reference forwarded in submit events.
    pub storage_ref: Option<String>,
}

/// Host-provided comment summary anchored to a markdown source line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownLineComment {
    /// One-indexed source line number.
    pub line: usize,
    /// Stable hash for the source line content.
    pub line_hash: String,
    /// Snapshot of the source line text.
    pub line_text: String,
    /// Number of comments attached to the line.
    pub comment_count: usize,
    /// Host-provided comment text to display when reopening the popup.
    pub comment_text: Option<String>,
}

/// Transient state for the multiline markdown comment popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommentPopupState {
    /// Whether the popup is currently active.
    pub active: bool,
    /// One-indexed source line currently targeted by the popup.
    pub line: usize,
    /// Stable hash for the targeted line text.
    pub line_hash: String,
    /// Snapshot of the targeted line text.
    pub line_text: String,
    /// Multiline draft comment text.
    pub buffer: String,
}

impl Default for CommentPopupState {
    /// Creates an inactive comment popup state.
    fn default() -> Self {
        Self {
            active: false,
            line: 1,
            line_hash: String::new(),
            line_text: String::new(),
            buffer: String::new(),
        }
    }
}

impl CommentPopupState {
    /// Opens the popup for the provided line anchor.
    pub fn open(&mut self, line: usize, line_hash: String, line_text: String) {
        self.active = true;
        self.line = line;
        self.line_hash = line_hash;
        self.line_text = line_text;
        self.buffer.clear();
    }

    /// Closes the popup and clears the draft buffer.
    pub fn close(&mut self) {
        self.active = false;
        self.buffer.clear();
    }

    /// Returns true when the popup is active for the provided source line.
    pub fn is_active_for_line(&self, line: usize) -> bool {
        self.active && self.line == line
    }
}
