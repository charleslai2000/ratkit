use super::{expand_tabs, pair_modified_lines_with_tab_width};
use crate::widgets::code_diff::code_diff::types::{
    ChangeType, DiffHunk, DiffLine, DiffLineCell, InlineSegment, SideBySideRow,
};

/// Builds render-ready side-by-side rows from parsed diff hunks.
pub fn build_side_by_side_rows(hunks: &[DiffHunk]) -> Vec<SideBySideRow> {
    build_side_by_side_rows_with_tab_width(hunks, 4)
}

/// Builds render-ready side-by-side rows using a configured tab width.
pub fn build_side_by_side_rows_with_tab_width(
    hunks: &[DiffHunk],
    tab_width: usize,
) -> Vec<SideBySideRow> {
    let mut rows = Vec::new();

    for hunk in hunks {
        rows.push(SideBySideRow::hunk_header(hunk.header.clone()));
        append_hunk_rows(&mut rows, &hunk.lines, tab_width);
    }

    rows
}

fn append_hunk_rows(rows: &mut Vec<SideBySideRow>, lines: &[DiffLine], tab_width: usize) {
    let mut index = 0;
    while index < lines.len() {
        if lines[index].is_removed() {
            index = append_removed_and_added_run(rows, lines, index, tab_width);
        } else if lines[index].is_added() {
            rows.push(added_row(&lines[index], tab_width));
            index += 1;
        } else if lines[index].is_context() {
            rows.push(context_row(&lines[index], tab_width));
            index += 1;
        } else {
            index += 1;
        }
    }
}

fn append_removed_and_added_run(
    rows: &mut Vec<SideBySideRow>,
    lines: &[DiffLine],
    start: usize,
    tab_width: usize,
) -> usize {
    let mut split = start;
    while split < lines.len() && lines[split].is_removed() {
        split += 1;
    }

    let mut end = split;
    while end < lines.len() && lines[end].is_added() {
        end += 1;
    }

    if split == end {
        for line in &lines[start..split] {
            rows.push(removed_row(line, tab_width));
        }
    } else {
        rows.extend(pair_modified_lines_with_tab_width(
            &lines[start..split],
            &lines[split..end],
            tab_width,
        ));
    }

    end
}

fn context_row(line: &DiffLine, tab_width: usize) -> SideBySideRow {
    let old_cell = cell_with_tab_width(line, line.old_line_num, tab_width);
    let new_cell = cell_with_tab_width(line, line.new_line_num, tab_width);
    SideBySideRow::new(Some(old_cell), Some(new_cell), ChangeType::Context)
}

fn added_row(line: &DiffLine, tab_width: usize) -> SideBySideRow {
    SideBySideRow::new(
        None,
        Some(cell_with_tab_width(line, line.new_line_num, tab_width)),
        ChangeType::Added,
    )
}

fn removed_row(line: &DiffLine, tab_width: usize) -> SideBySideRow {
    SideBySideRow::new(
        Some(cell_with_tab_width(line, line.old_line_num, tab_width)),
        None,
        ChangeType::Removed,
    )
}

fn cell_with_tab_width(
    line: &DiffLine,
    line_number: Option<usize>,
    tab_width: usize,
) -> DiffLineCell {
    let mut cell = DiffLineCell::from_diff_line(line, line_number);
    cell.content = expand_tabs(&cell.content, tab_width);
    cell.inline_segments = vec![InlineSegment::new(cell.content.clone(), false)];
    cell
}
