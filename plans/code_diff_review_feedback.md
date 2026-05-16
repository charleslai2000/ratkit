# Code Diff Review Feedback

## Review Scope

Compared the current unstaged `code-diff` changes against `plans/code_diff.md` after the requested fixes.

Reviewed files:

- `Cargo.toml`
- `src/widgets/code_diff/mod.rs`
- `src/widgets/code_diff/code_diff/mod.rs`
- `src/widgets/code_diff/code_diff/foundation.rs`
- `src/widgets/code_diff/code_diff/widget/mod.rs`
- `src/widgets/code_diff/code_diff/parse/*`
- `src/widgets/code_diff/code_diff/layout/*`
- `src/widgets/code_diff/code_diff/types/*`
- `tests/code_diff_parser_layout_tests.rs`
- `tests/code_diff_layout_tests.rs`

Validation run:

```bash
cargo fmt --check
cargo test --features code-diff
```

Result: both passed. `cargo test --features code-diff` reported 61 passed across 13 suites, with existing repository warnings.

## Current Assessment

The fixes address the previous Phase 1 + Phase 2 critical feedback. This change is now correctly scoped as a parser/layout foundation PR, not a full rendered diff viewer PR.

The remaining missing features are later phases from `plans/code_diff.md`, not blockers for a Phase 1 + Phase 2 foundation merge.

## Previously Critical Items

### 1. `CodeDiff` still does not render a diff

Status: **Resolved for current scope**.

The module docs now state this is only the Phase 1 + Phase 2 foundation and that real rendering is a later Phase 3 concern.

No further fix needed for this PR if reviewers agree this PR is intentionally parser/layout only.

### 2. `get_git_diff` silently returns an empty string

Status: **Resolved**.

`get_git_diff` now creates a pure unified text diff using `similar::TextDiff` and performs no VCS IO. Equal inputs still return an empty string, which is acceptable because there is no diff.

Follow-up recommendation:

- Consider renaming this helper later because `get_git_diff` sounds like repository/git IO, while the implementation is now pure text diff generation.

### 3. Parser loses file paths for standard unified diffs without `diff --git`

Status: **Resolved**.

The parser now handles `---` / `+++` headers before the first hunk and has regression coverage in `parses_standard_unified_diff_without_git_header`.

### 4. Path parsing is not robust for filenames with spaces or git quoting

Status: **Resolved for common cases**.

`parse_file_header` now supports paths with spaces and quoted git paths, with regression coverage in `parses_git_headers_with_spaces_and_quotes`.

Follow-up recommendation:

- Add future tests for C-style/octal quoted git path escapes if this parser needs to support non-UTF-8 or unusual filenames.

### 5. Inline similarity check does not match the plan

Status: **Resolved**.

Inline similarity now uses unchanged spans from `similar::TextDiff` instead of same-index character comparison. Regression coverage was added for inserted prefixes.

### 6. `expand_tabs` exists but is not used by row building

Status: **Resolved**.

`build_side_by_side_rows_with_tab_width` and `pair_modified_lines_with_tab_width` now apply tab expansion. Regression coverage was added for configured tab width.

## Remaining Missing Planned Features

These are still absent, but they belong to later phases unless this PR's scope changes.

1. Real unified and side-by-side rendering.
2. Snapshot tests using `ratatui::backend::TestBackend`.
3. `CodeDiffState` for selected file, scroll, hunk focus, panel focus, selection, and viewed files.
4. Typed `CodeDiffEvent` contract.
5. Hunk navigation and file navigation.
6. File tree adapter behind a feature flag.
7. Review/annotation event contract.
8. Optional highlight provider abstraction.
9. `NoHighlightProvider` default.
10. `code-diff-syntect`, `code-diff-tree-sitter`, `code-diff-file-tree`, and `code-diff-review` feature flags.
11. Updated examples demonstrating parser/layout use.
12. README/API docs explaining state ownership and host integration.

## Remaining Test Coverage Gaps

These are follow-up coverage items, not current blockers for Phase 1 + Phase 2.

1. Standard unified headers with timestamps, such as `--- file\tdate` / `+++ file\tdate`.
2. C-style/octal quoted git path escapes.
3. Rename-only diffs with no hunks.
4. Binary patch variants beyond `Binary files ... differ` and `GIT binary patch`.
5. Empty-file additions and deletions from pure unified text diff generation.
6. Rendering snapshots once Phase 3 starts.

## Process Issues

The working tree still contains many unrelated `markdown_preview` changes. Keep the `code-diff` work isolated from markdown-comment work during review and merge.

## Recommended Next Step

Treat this as a Phase 1 + Phase 2 foundation change. The next PR should start Phase 3 by adding real `CodeDiff` rendering and snapshot tests, using the parsed `DiffFile` and `SideBySideRow` data now provided here.
