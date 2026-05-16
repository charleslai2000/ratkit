use crate::widgets::markdown_preview::widgets::markdown_widget::state::{
    CommentPopupConfig, MarkdownLineComment,
};
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::MarkdownWidget;

impl<'a> MarkdownWidget<'a> {
    /// Sets the comment popup config used by the widget.
    pub fn with_comment_popup_config(mut self, config: CommentPopupConfig) -> Self {
        self.comment_popup_config = config;
        self
    }

    /// Replaces the comment popup config used by the widget.
    pub fn set_comment_popup_config(&mut self, config: CommentPopupConfig) {
        self.comment_popup_config = config;
    }

    /// Sets the host-provided line comment summaries for marker rendering.
    pub fn with_line_comments(mut self, comments: Vec<MarkdownLineComment>) -> Self {
        self.line_comments = comments;
        self
    }

    /// Replaces the host-provided line comment summaries for marker rendering.
    pub fn set_line_comments(&mut self, comments: Vec<MarkdownLineComment>) {
        self.line_comments = comments;
    }
}

/// Returns a stable FNV-1a hash for markdown line anchors.
pub fn line_content_hash(line_text: &str) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x00000100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in line_text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("{hash:016x}")
}

/// Returns the one-indexed source line for a one-indexed visual line.
pub fn source_line_for_visual_line(widget: &MarkdownWidget<'_>, visual_line: usize) -> usize {
    widget
        .cache
        .render
        .as_ref()
        .and_then(|cache| {
            cache
                .line_source_lines
                .get(visual_line.saturating_sub(1))
                .copied()
        })
        .unwrap_or(visual_line.max(1))
}

/// Returns the source text for a one-indexed markdown source line.
pub fn source_line_text(content: &str, source_line: usize) -> String {
    content
        .lines()
        .nth(source_line.saturating_sub(1))
        .unwrap_or("")
        .to_string()
}

/// Returns the current comment anchor as source line, line hash, and line text.
pub fn current_comment_anchor(widget: &MarkdownWidget<'_>) -> (usize, String, String) {
    let source_line = source_line_for_visual_line(widget, widget.scroll.current_line);
    let line_text = source_line_text(&widget.content, source_line);
    let line_hash = line_content_hash(&line_text);
    (source_line, line_hash, line_text)
}

/// Returns host-provided comment text for a source line when it is still anchored.
pub fn comment_text_for_line(
    comments: &[MarkdownLineComment],
    content: &str,
    source_line: usize,
) -> Option<String> {
    matching_comment_for_line(comments, content, source_line)
        .and_then(|comment| comment.comment_text.clone())
}

/// Returns true when a host-provided comment still matches current source content.
pub fn comment_matches_line(comment: &MarkdownLineComment, content: &str) -> bool {
    let line_text = source_line_text(content, comment.line);
    line_content_hash(&line_text) == comment.line_hash && line_text == comment.line_text
}

/// Finds a matching host-provided comment summary for the source line.
pub fn matching_comment_for_line<'a>(
    comments: &'a [MarkdownLineComment],
    content: &str,
    source_line: usize,
) -> Option<&'a MarkdownLineComment> {
    comments
        .iter()
        .find(|comment| comment.line == source_line && comment.comment_count > 0)
        .filter(|comment| comment_matches_line(comment, content))
}
