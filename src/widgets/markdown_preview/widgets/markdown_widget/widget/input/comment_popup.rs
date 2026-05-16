use crate::widgets::markdown_preview::widgets::markdown_widget::foundation::events::MarkdownEvent;
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::features::comments::{
    comment_text_for_line, current_comment_anchor,
};
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::{
    MarkdownWidget, MarkdownWidgetMode,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

impl MarkdownWidget<'_> {
    /// Handles a key event while the comment popup is active.
    pub(crate) fn handle_comment_popup_key(&mut self, key: KeyEvent) -> MarkdownEvent {
        match key.code {
            KeyCode::Enter => self.submit_comment_popup(),
            KeyCode::Esc => {
                let line = self.comment_popup.line;
                self.close_comment_popup();
                MarkdownEvent::CommentPopupToggled {
                    active: false,
                    line,
                }
            }
            KeyCode::Backspace => {
                self.comment_popup.buffer.pop();
                MarkdownEvent::CommentPopupChanged {
                    line: self.comment_popup.line,
                }
            }
            KeyCode::Char(c) if character_input_allowed(key.modifiers) => {
                self.comment_popup.buffer.push(c);
                MarkdownEvent::CommentPopupChanged {
                    line: self.comment_popup.line,
                }
            }
            _ => MarkdownEvent::None,
        }
    }

    /// Toggles the popup for the current markdown source line.
    pub(crate) fn toggle_comment_popup_for_current_line(&mut self) -> MarkdownEvent {
        let (line, line_hash, line_text) = current_comment_anchor(self);
        if self.comment_popup.is_active_for_line(line) {
            self.close_comment_popup();
            return MarkdownEvent::CommentPopupToggled {
                active: false,
                line,
            };
        }
        let existing_comment_text = comment_text_for_line(&self.line_comments, &self.content, line);
        self.comment_popup.open(line, line_hash, line_text);
        if let Some(comment_text) = existing_comment_text {
            self.comment_popup.buffer = comment_text;
        }
        self.mode = MarkdownWidgetMode::CommentPopup;
        MarkdownEvent::CommentPopupToggled { active: true, line }
    }

    /// Closes the comment popup and returns to normal mode.
    pub(crate) fn close_comment_popup(&mut self) {
        self.comment_popup.close();
        self.mode = MarkdownWidgetMode::Normal;
    }

    /// Submits the current popup draft as a typed markdown event.
    fn submit_comment_popup(&mut self) -> MarkdownEvent {
        let comment_text = self.comment_popup.buffer.clone();
        let line = self.comment_popup.line;
        let line_hash = self.comment_popup.line_hash.clone();
        let line_text = self.comment_popup.line_text.clone();
        let storage_ref = self.comment_popup_config.storage_ref.clone();
        self.close_comment_popup();
        MarkdownEvent::CommentSubmitted {
            line,
            line_hash,
            line_text,
            comment_text,
            storage_ref,
        }
    }
}

/// Returns true when key modifiers represent printable text input.
fn character_input_allowed(modifiers: KeyModifiers) -> bool {
    !modifiers.contains(KeyModifiers::CONTROL) && !modifiers.contains(KeyModifiers::ALT)
}
