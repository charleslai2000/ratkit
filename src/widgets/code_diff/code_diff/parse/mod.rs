//! Pure parsers for unified diff text.

mod parse_file_header;
mod parse_hunk_header;
mod parse_unified_diff;

pub use parse_file_header::{normalize_diff_path, parse_file_header, ParsedFileHeader};
pub use parse_hunk_header::{parse_hunk_header, ParsedHunkHeader};
pub use parse_unified_diff::parse_unified_diff;
