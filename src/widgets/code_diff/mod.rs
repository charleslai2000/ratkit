//! Code diff viewer for ratatui.

pub mod code_diff;

pub use code_diff::{
    ChangeType, CodeDiff, DiffConfig, DiffFile, DiffFileStatus, DiffHunk, DiffLine, DiffLineCell,
    DiffLineKind, DiffStyle, InlineSegment, SideBySideRow,
};
