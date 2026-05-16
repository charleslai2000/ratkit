//! Core data types for parsing and laying out code diffs.

mod change_type;
mod diff_file;
mod diff_file_status;
mod diff_hunk;
mod diff_line;
mod diff_line_cell;
mod diff_line_kind;
mod diff_style;
mod inline_segment;
mod side_by_side_row;

pub use change_type::ChangeType;
pub use diff_file::DiffFile;
pub use diff_file_status::DiffFileStatus;
pub use diff_hunk::DiffHunk;
pub use diff_line::DiffLine;
pub use diff_line_cell::DiffLineCell;
pub use diff_line_kind::DiffLineKind;
pub use diff_style::DiffStyle;
pub use inline_segment::InlineSegment;
pub use side_by_side_row::SideBySideRow;
