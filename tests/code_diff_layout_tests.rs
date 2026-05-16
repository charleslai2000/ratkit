#![cfg(feature = "code-diff")]

use ratkit::widgets::code_diff::code_diff::layout::{
    build_side_by_side_rows, build_side_by_side_rows_with_tab_width, compute_inline_segments,
};
use ratkit::widgets::code_diff::code_diff::parse::parse_unified_diff;
use ratkit::widgets::code_diff::code_diff::types::ChangeType;

#[test]
fn side_by_side_rows_pair_modified_lines_with_inline_segments() {
    let diff = concat!(
        "diff --git a/src/lib.rs b/src/lib.rs\n",
        "--- a/src/lib.rs\n",
        "+++ b/src/lib.rs\n",
        "@@ -1,3 +1,3 @@\n",
        " unchanged\n",
        "-let color = red;\n",
        "+let color = blue;\n",
        "+let enabled = true;\n",
    );
    let files = parse_unified_diff(diff);

    let rows = build_side_by_side_rows(&files[0].hunks);

    assert_eq!(rows[0].change_type, ChangeType::HunkHeader);
    assert_eq!(rows[2].change_type, ChangeType::Modified);
    assert_eq!(
        rows[2].old_line.as_ref().unwrap().content,
        "let color = red;"
    );
    assert_eq!(
        rows[2].new_line.as_ref().unwrap().content,
        "let color = blue;"
    );
    assert!(rows[2]
        .old_line
        .as_ref()
        .unwrap()
        .inline_segments
        .iter()
        .any(|segment| segment.emphasized && segment.text.contains("red")));
    assert_eq!(rows[3].change_type, ChangeType::Added);
    assert!(rows[3].old_line.is_none());
}

#[test]
fn inline_similarity_uses_unchanged_spans_for_inserted_prefixes() {
    let (old_segments, new_segments) =
        compute_inline_segments("let color = blue;", "pub let color = blue;");

    assert!(new_segments
        .iter()
        .any(|segment| segment.emphasized && segment.text.contains("pub")));
    assert!(old_segments.iter().any(|segment| !segment.emphasized));
}

#[test]
fn side_by_side_rows_expand_tabs_with_configured_width() {
    let diff = concat!(
        "--- a.rs\n",
        "+++ a.rs\n",
        "@@ -1 +1 @@\n",
        "-let\told\n",
        "+let\tnew\n",
    );
    let files = parse_unified_diff(diff);

    let rows = build_side_by_side_rows_with_tab_width(&files[0].hunks, 2);

    assert_eq!(rows[1].old_line.as_ref().unwrap().content, "let old");
    assert_eq!(rows[1].new_line.as_ref().unwrap().content, "let new");
}
