use std::path::PathBuf;

/// Returns the `/tmp` file path used by the markdown preview demo comment store.
pub fn comment_store_path() -> PathBuf {
    PathBuf::from("/tmp/ratkit-markdown-preview-demo-comments.json")
}
