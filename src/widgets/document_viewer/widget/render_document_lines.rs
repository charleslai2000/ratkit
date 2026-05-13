//! Line renderer for normalized document viewports.

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
};

use crate::widgets::document_viewer::{
    extensions::{gutter_width, is_line_selected, render_gutter, selected_line_style},
    foundation::DocumentLine,
    state::{DisplaySettings, ScrollState, SelectionState},
};

const CURRENT_LINE_BG: Color = Color::Rgb(38, 52, 63);

/// Renders visible document lines into the target area.
pub fn render_document_lines(
    area: Rect,
    buf: &mut Buffer,
    lines: &[DocumentLine],
    scroll: &ScrollState,
    display: &DisplaySettings,
    selection: Option<&SelectionState>,
) {
    if area.is_empty() {
        return;
    }

    let gutter_width = gutter_width(lines.len(), display);
    let mut local_scroll = scroll.clone();
    local_scroll.ensure_current_visible(area.height as usize);

    for (row, line_index) in local_scroll.visible_range(area.height as usize).enumerate() {
        render_line(
            area,
            buf,
            lines,
            line_index,
            row,
            gutter_width,
            &local_scroll,
            display,
            selection,
        );
    }
}

/// Renders one visible line.
fn render_line(
    area: Rect,
    buf: &mut Buffer,
    lines: &[DocumentLine],
    line_index: usize,
    row: usize,
    gutter_width: usize,
    scroll: &ScrollState,
    display: &DisplaySettings,
    selection: Option<&SelectionState>,
) {
    let Some(document_line) = lines.get(line_index) else {
        return;
    };
    let y = area.y + row as u16;
    let is_current = line_index == scroll.current_line_index();
    let is_selected = is_line_selected(selection, line_index);
    buf.set_style(
        Rect {
            y,
            height: 1,
            ..area
        },
        line_style(is_current, is_selected, display),
    );
    render_gutter(area.x, y, gutter_width, line_index, scroll, display, buf);
    let line = Line::from(document_line.spans.clone());
    buf.set_line(
        area.x + gutter_width as u16,
        y,
        &line,
        area.width.saturating_sub(gutter_width as u16),
    );
}

/// Returns the row background style for a rendered line.
fn line_style(is_current: bool, is_selected: bool, display: &DisplaySettings) -> Style {
    if is_selected {
        selected_line_style(true)
    } else if is_current && display.highlight_current_line {
        Style::default().bg(CURRENT_LINE_BG)
    } else {
        Style::default()
    }
}
