use super::DiffLine;

/// A parsed hunk from a unified diff file entry.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiffHunk {
    /// First line number in the old file.
    pub old_start: usize,
    /// Number of old-file lines covered by the hunk.
    pub old_count: usize,
    /// First line number in the new file.
    pub new_start: usize,
    /// Number of new-file lines covered by the hunk.
    pub new_count: usize,
    /// Parsed hunk body lines.
    pub lines: Vec<DiffLine>,
    /// Original hunk header text.
    pub header: String,
}

impl DiffHunk {
    /// Creates an empty hunk from explicit source ranges.
    pub fn new(old_start: usize, old_count: usize, new_start: usize, new_count: usize) -> Self {
        Self {
            old_start,
            old_count,
            new_start,
            new_count,
            lines: Vec::new(),
            header: format!(
                "@@ -{},{} +{},{} @@",
                old_start, old_count, new_start, new_count
            ),
        }
    }

    /// Creates a hunk by parsing a unified diff hunk header.
    pub fn from_header(header: &str) -> Self {
        crate::widgets::code_diff::code_diff::parse::parse_hunk_header(header)
            .map(|parsed| {
                let mut hunk = Self::new(
                    parsed.old_start,
                    parsed.old_count,
                    parsed.new_start,
                    parsed.new_count,
                );
                hunk.header = header.to_string();
                hunk
            })
            .unwrap_or_else(|| Self::new(1, 0, 1, 0))
    }

    /// Adds a parsed line to this hunk.
    pub fn add_line(&mut self, line: DiffLine) {
        self.lines.push(line);
    }

    /// Returns parsed hunk body lines.
    pub fn lines(&self) -> &[DiffLine] {
        &self.lines
    }

    /// Counts added lines in the hunk body.
    pub fn added_count(&self) -> usize {
        self.lines.iter().filter(|line| line.is_added()).count()
    }

    /// Counts removed lines in the hunk body.
    pub fn removed_count(&self) -> usize {
        self.lines.iter().filter(|line| line.is_removed()).count()
    }

    /// Counts all body lines in the hunk.
    pub fn total_lines(&self) -> usize {
        self.lines.len()
    }
}
