//! Line-number gutter rendering for document viewers.

use ratatui::{
    buffer::Buffer,
    style::{Color, Style},
};

use crate::widgets::document_viewer::state::{DisplaySettings, ScrollState};

/// Calculates the line-number gutter width for a document.
pub fn gutter_width(line_count: usize, display: &DisplaySettings) -> usize {
    if !display.show_line_numbers {
        return 0;
    }
    line_count.max(1).to_string().len() + 5
}

/// Renders one line-number gutter cell.
pub fn render_gutter(
    x: u16,
    y: u16,
    width: usize,
    line_index: usize,
    scroll: &ScrollState,
    display: &DisplaySettings,
    buf: &mut Buffer,
) {
    if width == 0 {
        return;
    }
    let current_line = scroll.current_line_index();
    let number = if display.relative_line_numbers && line_index != current_line {
        line_index.abs_diff(current_line).to_string()
    } else {
        (line_index + 1).to_string()
    };
    let number_width = width.saturating_sub(3);
    let text = format!("{number:>number_width$} │ ");
    buf.set_stringn(x, y, text, width, Style::default().fg(Color::DarkGray));
}
