#![cfg(feature = "code-diff")]

use ratkit::widgets::code_diff::code_diff::get_git_diff;
use ratkit::widgets::code_diff::code_diff::parse::{
    parse_file_header, parse_hunk_header, parse_unified_diff,
};
use ratkit::widgets::code_diff::code_diff::types::DiffFileStatus;
use ratkit::widgets::code_diff::CodeDiff;

#[test]
fn parses_hunk_headers_with_optional_counts() {
    let header = parse_hunk_header("@@ -7 +8,2 @@ fn demo").expect("header should parse");

    assert_eq!(header.old_start, 7);
    assert_eq!(header.old_count, 1);
    assert_eq!(header.new_start, 8);
    assert_eq!(header.new_count, 2);
}

#[test]
fn parses_multi_file_unified_diff_metadata() {
    let diff = concat!(
        "diff --git a/src/a.rs b/src/a.rs\n",
        "--- a/src/a.rs\n",
        "+++ b/src/a.rs\n",
        "@@ -1 +1 @@\n",
        "-old\n",
        "+new\n",
        "diff --git a/src/new.rs b/src/new.rs\n",
        "new file mode 100644\n",
        "--- /dev/null\n",
        "+++ b/src/new.rs\n",
        "@@ -0,0 +1 @@\n",
        "+created\n",
        "diff --git a/src/old.rs b/src/old.rs\n",
        "deleted file mode 100644\n",
        "--- a/src/old.rs\n",
        "+++ /dev/null\n",
        "@@ -1 +0,0 @@\n",
        "-removed\n",
        "diff --git a/src/before.rs b/src/after.rs\n",
        "similarity index 90%\n",
        "rename from src/before.rs\n",
        "rename to src/after.rs\n",
        "--- a/src/before.rs\n",
        "+++ b/src/after.rs\n",
        "@@ -1 +1 @@\n",
        " same\n",
        "diff --git a/assets/logo.png b/assets/logo.png\n",
        "Binary files a/assets/logo.png and b/assets/logo.png differ\n",
    );

    let files = parse_unified_diff(diff);

    assert_eq!(files.len(), 5);
    assert_eq!(files[0].path, "src/a.rs");
    assert_eq!(files[0].status, DiffFileStatus::Modified);
    assert_eq!(files[1].status, DiffFileStatus::Added);
    assert_eq!(files[2].status, DiffFileStatus::Deleted);
    assert_eq!(files[3].old_path.as_deref(), Some("src/before.rs"));
    assert_eq!(files[3].status, DiffFileStatus::Renamed);
    assert!(files[4].is_binary);
}

#[test]
fn code_diff_from_unified_diff_keeps_multi_file_hunks() {
    let diff = concat!(
        "diff --git a/one.rs b/one.rs\n",
        "--- a/one.rs\n",
        "+++ b/one.rs\n",
        "@@ -1 +1 @@\n",
        "-one\n",
        "+two\n",
        "diff --git a/two.rs b/two.rs\n",
        "--- a/two.rs\n",
        "+++ b/two.rs\n",
        "@@ -3 +3 @@\n",
        "-three\n",
        "+four\n",
    );

    let widget = CodeDiff::from_unified_diff(diff);

    assert_eq!(widget.hunks().len(), 1);
    assert_eq!(widget.file_diffs.len(), 2);
    assert_eq!(widget.file_path.as_deref(), Some("one.rs"));
}

#[test]
fn parses_standard_unified_diff_without_git_header() {
    let diff = concat!(
        "--- old name.rs\n",
        "+++ new name.rs\n",
        "@@ -1 +1 @@\n",
        "-old\n",
        "+new\n",
    );

    let files = parse_unified_diff(diff);

    assert_eq!(files.len(), 1);
    assert_eq!(files[0].path, "new name.rs");
    assert_eq!(files[0].old_path.as_deref(), Some("old name.rs"));
    assert_eq!(files[0].hunks[0].lines[0].old_line_num, Some(1));
    assert_eq!(files[0].hunks[0].lines[1].new_line_num, Some(1));
}

#[test]
fn parses_git_headers_with_spaces_and_quotes() {
    let spaced = parse_file_header("diff --git a/my file.rs b/my file.rs")
        .expect("spaced path should parse");
    let quoted = parse_file_header("diff --git \"a/old file.rs\" \"b/new file.rs\"")
        .expect("quoted path should parse");

    assert_eq!(spaced.old_path, "my file.rs");
    assert_eq!(spaced.new_path, "my file.rs");
    assert_eq!(quoted.old_path, "old file.rs");
    assert_eq!(quoted.new_path, "new file.rs");
}

#[test]
fn parser_ignores_no_newline_markers_without_line_number_drift() {
    let diff = concat!(
        "--- a.rs\n",
        "+++ a.rs\n",
        "@@ -1,2 +1,2 @@\n",
        " same\n",
        "-old\n",
        "\\ No newline at end of file\n",
        "+new\n",
    );

    let files = parse_unified_diff(diff);

    assert_eq!(files[0].hunks[0].lines.len(), 3);
    assert_eq!(files[0].hunks[0].lines[2].new_line_num, Some(2));
}

#[test]
fn parser_keeps_line_numbers_across_multiple_hunks() {
    let diff = concat!(
        "--- a.rs\n",
        "+++ a.rs\n",
        "@@ -1 +1 @@\n",
        "-one\n",
        "+two\n",
        "@@ -10 +10 @@\n",
        "-ten\n",
        "+eleven\n",
    );

    let files = parse_unified_diff(diff);

    assert_eq!(files[0].hunks[0].lines[0].old_line_num, Some(1));
    assert_eq!(files[0].hunks[1].lines[0].old_line_num, Some(10));
    assert_eq!(files[0].hunks[1].lines[1].new_line_num, Some(10));
}

#[test]
fn pure_get_git_diff_returns_changed_text_diff() {
    let diff = get_git_diff("old\n", "new\n");

    assert!(diff.contains("@@ -1,1 +1,1 @@"));
    assert!(diff.contains("-old"));
    assert!(diff.contains("+new"));
}
