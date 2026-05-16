use crate::widgets::markdown_preview::widgets::markdown_widget::widget::features::comments::matching_comment_for_line;
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::MarkdownWidget;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph, Widget};
use unicode_width::UnicodeWidthStr;

const COMMENT_MARKER: &str = "●";
const COMMENT_GUTTER_WIDTH: usize = 2;
const POPUP_MAX_HEIGHT: u16 = 8;

/// Returns the width reserved for the comment gutter.
pub fn comment_gutter_width(has_comments: bool) -> usize {
    if has_comments {
        COMMENT_GUTTER_WIDTH
    } else {
        0
    }
}

/// Returns the marker to render for a visible markdown source line.
pub fn marker_for_source_line(
    widget: &MarkdownWidget<'_>,
    source_line: usize,
) -> Option<&'static str> {
    matching_comment_for_line(&widget.line_comments, &widget.content, source_line)
        .map(|_| COMMENT_MARKER)
}

/// Applies a right-gutter comment marker to a rendered line.
pub fn apply_comment_gutter(
    line: Line<'static>,
    marker: Option<&str>,
    content_width: usize,
) -> Line<'static> {
    if marker.is_none() && content_width == 0 {
        return line;
    }
    let marker_text = marker.unwrap_or(" ");
    let line_width = line
        .spans
        .iter()
        .map(|span| span.content.as_ref().width())
        .sum::<usize>();
    let marker_width = marker_text.width();
    let target_width = content_width.saturating_sub(marker_width);
    let padding = target_width.saturating_sub(line_width);
    let mut spans = line.spans;
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(
        marker_text.to_string(),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    ));
    Line::from(spans)
}

/// Renders the active comment popup inside the markdown viewport.
pub fn render_comment_popup(
    widget: &MarkdownWidget<'_>,
    area: Rect,
    buf: &mut ratatui::buffer::Buffer,
) {
    if !widget.comment_popup.active || area.is_empty() {
        return;
    }
    let popup_area = comment_popup_area(widget, area);
    Clear.render(popup_area, buf);
    let title = format!(" Comment line {} ", widget.comment_popup.line);
    let paragraph = Paragraph::new(comment_popup_input_line(&widget.comment_popup.buffer))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray))
                .title(title),
        )
        .style(Style::default().fg(Color::White).bg(Color::Rgb(24, 24, 32)));
    paragraph.render(popup_area, buf);
}

/// Calculates the popup rectangle below the selected line when possible.
fn comment_popup_area(widget: &MarkdownWidget<'_>, area: Rect) -> Rect {
    let row = widget
        .scroll
        .current_line
        .saturating_sub(widget.scroll.scroll_offset)
        .saturating_sub(1) as u16;
    let desired_height = desired_popup_height(&widget.comment_popup.buffer);
    let width = area.width.saturating_sub(4).max(20).min(area.width);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let below_y = area.y.saturating_add(row).saturating_add(1);
    let below_fits = below_y.saturating_add(desired_height) <= area.y.saturating_add(area.height);
    let y = if below_fits {
        below_y
    } else {
        area.y
            .saturating_add(row)
            .saturating_sub(desired_height)
            .max(area.y)
    };
    Rect {
        x,
        y,
        width,
        height: desired_height.min(area.height),
    }
}

/// Builds the popup input line with a visible cursor at the draft end.
fn comment_popup_input_line(buffer: &str) -> Line<'static> {
    Line::from(vec![
        Span::raw(buffer.to_string()),
        Span::styled(
            "█".to_string(),
            Style::default().fg(Color::White).bg(Color::Rgb(80, 80, 96)),
        ),
    ])
}

/// Returns the popup height needed for the current draft buffer.
fn desired_popup_height(buffer: &str) -> u16 {
    let body_lines = buffer.lines().count().max(1) as u16;
    body_lines.saturating_add(2).min(POPUP_MAX_HEIGHT)
}
