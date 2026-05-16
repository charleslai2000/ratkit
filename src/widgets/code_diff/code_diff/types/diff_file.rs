use super::{DiffFileStatus, DiffHunk};

/// A parsed file entry in a unified diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffFile {
    /// The current path of the file.
    pub path: String,
    /// The previous path when the file was renamed or deleted.
    pub old_path: Option<String>,
    /// The file-level diff status.
    pub status: DiffFileStatus,
    /// Text hunks parsed for this file.
    pub hunks: Vec<DiffHunk>,
    /// Whether this file contains binary changes without text hunks.
    pub is_binary: bool,
}

impl DiffFile {
    /// Creates an empty modified file entry for the provided path.
    pub fn new(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            old_path: None,
            status: DiffFileStatus::Modified,
            hunks: Vec::new(),
            is_binary: false,
        }
    }

    /// Adds a parsed hunk to the file.
    pub fn add_hunk(&mut self, hunk: DiffHunk) {
        self.hunks.push(hunk);
    }
}
