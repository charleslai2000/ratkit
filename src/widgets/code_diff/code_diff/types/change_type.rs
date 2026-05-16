/// Describes how a rendered diff row changes between old and new content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChangeType {
    /// The row contains unchanged context.
    Context,
    /// The row exists only on the new side.
    Added,
    /// The row exists only on the old side.
    Removed,
    /// The row pairs an old line with its replacement.
    Modified,
    /// The row is a hunk metadata header.
    #[default]
    HunkHeader,
}
