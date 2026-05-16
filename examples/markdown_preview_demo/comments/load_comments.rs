use std::io;

use ratkit::widgets::markdown_preview::MarkdownLineComment;

use super::comment_store_path::comment_store_path;
use super::from_stored_markdown_line_comment::from_stored_markdown_line_comment;
use super::stored_markdown_line_comment::StoredMarkdownLineComment;

/// Loads markdown preview demo comments from the temp-file comment store.
pub fn load_comments() -> io::Result<Vec<MarkdownLineComment>> {
    let path = comment_store_path();
    if !path.exists() {
        return Ok(Vec::new());
    }

    let contents = std::fs::read_to_string(path)?;
    let stored_comments: Vec<StoredMarkdownLineComment> = serde_json::from_str(&contents)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    Ok(stored_comments
        .into_iter()
        .map(from_stored_markdown_line_comment)
        .collect())
}
