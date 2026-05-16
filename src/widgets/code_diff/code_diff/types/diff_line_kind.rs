/// Describes the kind of a single parsed diff line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffLineKind {
    /// An unchanged context line.
    Context,
    /// A line added by the diff.
    Added,
    /// A line removed by the diff.
    Removed,
    /// A hunk header line.
    #[default]
    HunkHeader,
}
