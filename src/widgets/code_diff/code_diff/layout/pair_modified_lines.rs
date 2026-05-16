use super::{compute_inline_segments, expand_tabs};
use crate::widgets::code_diff::code_diff::types::{
    ChangeType, DiffLine, DiffLineCell, SideBySideRow,
};

/// Pairs adjacent removed and added lines into aligned side-by-side rows.
pub fn pair_modified_lines(removed: &[DiffLine], added: &[DiffLine]) -> Vec<SideBySideRow> {
    pair_modified_lines_with_tab_width(removed, added, 4)
}

/// Pairs adjacent removed and added lines using a configured tab width.
pub fn pair_modified_lines_with_tab_width(
    removed: &[DiffLine],
    added: &[DiffLine],
    tab_width: usize,
) -> Vec<SideBySideRow> {
    let mut rows = Vec::new();
    let row_count = removed.len().max(added.len());

    for index in 0..row_count {
        let old_line = removed.get(index);
        let new_line = added.get(index);
        rows.push(match (old_line, new_line) {
            (Some(old_line), Some(new_line)) => modified_row(old_line, new_line, tab_width),
            (Some(old_line), None) => SideBySideRow::new(
                Some(cell_with_tab_width(
                    old_line,
                    old_line.old_line_num,
                    tab_width,
                )),
                None,
                ChangeType::Removed,
            ),
            (None, Some(new_line)) => SideBySideRow::new(
                None,
                Some(cell_with_tab_width(
                    new_line,
                    new_line.new_line_num,
                    tab_width,
                )),
                ChangeType::Added,
            ),
            (None, None) => unreachable!("row_count prevents empty pairs"),
        });
    }

    rows
}

fn modified_row(old_line: &DiffLine, new_line: &DiffLine, tab_width: usize) -> SideBySideRow {
    let old_content = expand_tabs(&old_line.content, tab_width);
    let new_content = expand_tabs(&new_line.content, tab_width);
    let (old_segments, new_segments) = compute_inline_segments(&old_content, &new_content);
    let mut old_cell = cell_with_tab_width(old_line, old_line.old_line_num, tab_width);
    let mut new_cell = cell_with_tab_width(new_line, new_line.new_line_num, tab_width);
    old_cell.inline_segments = old_segments;
    new_cell.inline_segments = new_segments;
    SideBySideRow::new(Some(old_cell), Some(new_cell), ChangeType::Modified)
}

fn cell_with_tab_width(
    line: &DiffLine,
    line_number: Option<usize>,
    tab_width: usize,
) -> DiffLineCell {
    let mut cell = DiffLineCell::from_diff_line(line, line_number);
    cell.content = expand_tabs(&cell.content, tab_width);
    cell.inline_segments = vec![
        crate::widgets::code_diff::code_diff::types::InlineSegment::new(
            cell.content.clone(),
            false,
        ),
    ];
    cell
}
