use ratkit::widgets::markdown_preview::MarkdownLineComment;

use super::stored_markdown_line_comment::StoredMarkdownLineComment;

/// Converts a widget comment anchor into its persisted demo representation.
pub fn to_stored_markdown_line_comment(comment: &MarkdownLineComment) -> StoredMarkdownLineComment {
    StoredMarkdownLineComment {
        line: comment.line,
        line_hash: comment.line_hash.clone(),
        line_text: comment.line_text.clone(),
        comment_count: comment.comment_count,
        comment_text: comment.comment_text.clone(),
    }
}
