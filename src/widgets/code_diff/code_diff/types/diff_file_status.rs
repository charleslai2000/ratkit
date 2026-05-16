/// Describes the file-level status represented by a diff file entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffFileStatus {
    /// The file has content changes.
    #[default]
    Modified,
    /// The file is newly created.
    Added,
    /// The file is removed.
    Deleted,
    /// The file path changed, possibly with content changes.
    Renamed,
    /// The file has binary changes without text hunks.
    Binary,
}
