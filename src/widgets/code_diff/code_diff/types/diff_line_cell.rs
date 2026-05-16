use super::{DiffLine, InlineSegment};

/// A render-ready cell for one side of a side-by-side diff row.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffLineCell {
    /// The original source line number for this side of the diff.
    pub line_number: Option<usize>,
    /// The line content without a diff prefix.
    pub content: String,
    /// Word-level spans used for inline change emphasis.
    pub inline_segments: Vec<InlineSegment>,
}

impl DiffLineCell {
    /// Creates a cell from a parsed diff line and side-specific line number.
    pub fn from_diff_line(line: &DiffLine, line_number: Option<usize>) -> Self {
        Self {
            line_number,
            content: line.content.clone(),
            inline_segments: vec![InlineSegment::new(line.content.clone(), false)],
        }
    }
}
