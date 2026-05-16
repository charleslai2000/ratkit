use serde::{Deserialize, Serialize};

/// Serializable representation of a demo markdown line comment.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredMarkdownLineComment {
    /// One-indexed markdown source line number.
    pub line: usize,
    /// Stable hash for the anchored source line text.
    pub line_hash: String,
    /// Snapshot of the anchored source line text.
    pub line_text: String,
    /// Number of comments attached to the source line.
    pub comment_count: usize,
    /// Persisted comment draft text shown when reopening the popup.
    pub comment_text: Option<String>,
}
