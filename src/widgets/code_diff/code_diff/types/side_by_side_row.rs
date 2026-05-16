use super::{ChangeType, DiffLineCell};

/// A render-ready row containing old and new side cells.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SideBySideRow {
    /// The old-side cell when the row has old content.
    pub old_line: Option<DiffLineCell>,
    /// The new-side cell when the row has new content.
    pub new_line: Option<DiffLineCell>,
    /// The row-level change classification.
    pub change_type: ChangeType,
    /// Hunk header text when this is a hunk metadata row.
    pub hunk_header: Option<String>,
}

impl SideBySideRow {
    /// Creates a row from optional old and new cells.
    pub fn new(
        old_line: Option<DiffLineCell>,
        new_line: Option<DiffLineCell>,
        change_type: ChangeType,
    ) -> Self {
        Self {
            old_line,
            new_line,
            change_type,
            hunk_header: None,
        }
    }

    /// Creates a hunk header row.
    pub fn hunk_header(header: impl Into<String>) -> Self {
        Self {
            old_line: None,
            new_line: None,
            change_type: ChangeType::HunkHeader,
            hunk_header: Some(header.into()),
        }
    }
}
