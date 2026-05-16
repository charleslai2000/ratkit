use std::io;

use ratkit::widgets::markdown_preview::MarkdownLineComment;

use super::comment_store_path::comment_store_path;
use super::to_stored_markdown_line_comment::to_stored_markdown_line_comment;

/// Saves markdown preview demo comments into the temp-file comment store.
pub fn save_comments(comments: &[MarkdownLineComment]) -> io::Result<()> {
    let path = comment_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let stored_comments = comments
        .iter()
        .map(to_stored_markdown_line_comment)
        .collect::<Vec<_>>();
    let contents = serde_json::to_string_pretty(&stored_comments)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    std::fs::write(path, contents)
}
