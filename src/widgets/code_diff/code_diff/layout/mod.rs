//! Pure layout builders for parsed code diffs.

mod build_side_by_side_rows;
mod compute_inline_segments;
mod expand_tabs;
mod pair_modified_lines;

pub use build_side_by_side_rows::{
    build_side_by_side_rows, build_side_by_side_rows_with_tab_width,
};
pub use compute_inline_segments::compute_inline_segments;
pub use expand_tabs::expand_tabs;
pub use pair_modified_lines::{pair_modified_lines, pair_modified_lines_with_tab_width};
