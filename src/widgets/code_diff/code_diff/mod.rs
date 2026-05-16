//! Code diff widget for displaying side-by-side diffs.
//!
//! This module currently implements the Phase 1 and Phase 2 foundation from
//! `plans/code_diff.md`: reusable parsing and side-by-side row layout. Real
//! unified and side-by-side row rendering remains a later Phase 3 concern.

pub mod foundation;
pub mod layout;
pub mod parse;
pub mod types;
pub mod widget;

pub use foundation::diff_config::DiffConfig;
pub use foundation::helpers::get_git_diff;
pub use types::{
    ChangeType, DiffFile, DiffFileStatus, DiffHunk, DiffLine, DiffLineCell, DiffLineKind,
    DiffStyle, InlineSegment, SideBySideRow,
};
pub use widget::CodeDiff;
