use ratkit::widgets::markdown_preview::MarkdownLineComment;

use super::stored_markdown_line_comment::StoredMarkdownLineComment;

/// Converts a persisted demo comment into the widget comment anchor type.
pub fn from_stored_markdown_line_comment(
    comment: StoredMarkdownLineComment,
) -> MarkdownLineComment {
    MarkdownLineComment {
        line: comment.line,
        line_hash: comment.line_hash,
        line_text: comment.line_text,
        comment_count: comment.comment_count,
        comment_text: comment.comment_text,
    }
}
