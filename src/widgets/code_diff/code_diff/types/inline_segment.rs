/// A word-level span within a side-by-side diff cell.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InlineSegment {
    /// The visible text contained in this segment.
    pub text: String,
    /// Whether this segment should receive inline change emphasis.
    pub emphasized: bool,
}

impl InlineSegment {
    /// Creates a segment with the provided text and emphasis state.
    pub fn new(text: impl Into<String>, emphasized: bool) -> Self {
        Self {
            text: text.into(),
            emphasized,
        }
    }
}
