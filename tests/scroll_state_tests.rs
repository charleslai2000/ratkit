#![cfg(any(feature = "code-widget", feature = "markdown-preview"))]

use ratkit::widgets::document_viewer::ScrollState;

#[test]
fn wheel_scroll_down_keeps_offset_after_visibility_adjustment() {
    let mut scroll = ScrollState::default();
    scroll.update_total_lines(100);
    scroll.viewport_height = 10;

    scroll.scroll_down(5);
    scroll.ensure_current_visible(10);

    assert_eq!(scroll.effective_offset(), 5);
    assert_eq!(scroll.current_line, 9);
}

#[test]
fn wheel_scroll_up_keeps_offset_after_visibility_adjustment() {
    let mut scroll = ScrollState::default();
    scroll.update_total_lines(100);
    scroll.viewport_height = 10;
    scroll.scroll_to_bottom();

    scroll.scroll_up(5);
    scroll.ensure_current_visible(10);

    assert_eq!(scroll.effective_offset(), 85);
    assert_eq!(scroll.current_line, 92);
}
