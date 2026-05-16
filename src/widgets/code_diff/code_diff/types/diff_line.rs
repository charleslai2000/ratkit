use super::DiffLineKind;

/// A single parsed line inside a unified diff hunk.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiffLine {
    /// The semantic type of this diff line.
    pub kind: DiffLineKind,
    /// The line text without the unified diff prefix.
    pub content: String,
    /// The old file line number when the line exists on the old side.
    pub old_line_num: Option<usize>,
    /// The new file line number when the line exists on the new side.
    pub new_line_num: Option<usize>,
}

impl DiffLine {
    /// Creates an unchanged context line.
    pub fn context(content: &str, old_num: usize, new_num: usize) -> Self {
        Self::new(DiffLineKind::Context, content, Some(old_num), Some(new_num))
    }

    /// Creates an added line.
    pub fn added(content: &str, new_num: usize) -> Self {
        Self::new(DiffLineKind::Added, content, None, Some(new_num))
    }

    /// Creates a removed line.
    pub fn removed(content: &str, old_num: usize) -> Self {
        Self::new(DiffLineKind::Removed, content, Some(old_num), None)
    }

    /// Creates a hunk header line.
    pub fn hunk_header(content: &str) -> Self {
        Self::new(DiffLineKind::HunkHeader, content, None, None)
    }

    /// Parses a raw unified diff line without assigning source line numbers.
    pub fn from_diff_line(line: &str) -> Option<Self> {
        match line.chars().next() {
            Some('+') if !line.starts_with("+++") => Some(Self::added(&line[1..], 0)),
            Some('-') if !line.starts_with("---") => Some(Self::removed(&line[1..], 0)),
            Some('@') => Some(Self::hunk_header(line)),
            Some(' ') => Some(Self::context(&line[1..], 0, 0)),
            Some('\t') => Some(Self::context(line, 0, 0)),
            _ => None,
        }
    }

    /// Returns true when this line is unchanged context.
    pub fn is_context(&self) -> bool {
        self.kind == DiffLineKind::Context
    }

    /// Returns true when this line was added.
    pub fn is_added(&self) -> bool {
        self.kind == DiffLineKind::Added
    }

    /// Returns true when this line was removed.
    pub fn is_removed(&self) -> bool {
        self.kind == DiffLineKind::Removed
    }

    /// Returns true when this line is a hunk header.
    pub fn is_hunk_header(&self) -> bool {
        self.kind == DiffLineKind::HunkHeader
    }

    /// Returns the unified diff prefix for this line kind.
    pub fn prefix(&self) -> &'static str {
        match self.kind {
            DiffLineKind::Context => " ",
            DiffLineKind::Added => "+",
            DiffLineKind::Removed => "-",
            DiffLineKind::HunkHeader => "@",
        }
    }

    /// Creates a line from explicit parts.
    pub fn new(
        kind: DiffLineKind,
        content: &str,
        old_line_num: Option<usize>,
        new_line_num: Option<usize>,
    ) -> Self {
        Self {
            kind,
            content: content.to_string(),
            old_line_num,
            new_line_num,
        }
    }
}
