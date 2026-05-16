#![cfg(feature = "markdown-preview")]

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers};
use insta::assert_snapshot;
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ratkit::widgets::markdown_preview::{
    CacheState, CollapseState, DisplaySettings, DoubleClickState, ExpandableState, GitStatsState,
    MarkdownEvent, MarkdownLineComment, MarkdownWidget, ScrollState, SelectionState, SourceState,
    VimState,
};

/// Creates a key-press event for markdown widget E2E tests.
fn key_event(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
    KeyEvent {
        code,
        modifiers,
        kind: KeyEventKind::Press,
        state: KeyEventState::NONE,
    }
}

/// Builds a markdown widget configured like the demo content area.
fn build_widget(markdown: &str) -> MarkdownWidget<'static> {
    build_widget_with_display(markdown, DisplaySettings::default())
}

/// Builds a markdown widget with custom display settings.
fn build_widget_with_display(markdown: &str, display: DisplaySettings) -> MarkdownWidget<'static> {
    let mut source = SourceState::default();
    source.set_source_string(markdown.to_string());

    let mut scroll = ScrollState::default();
    scroll.update_total_lines(markdown.lines().count().max(1));

    MarkdownWidget::new(
        markdown.to_string(),
        scroll,
        source,
        CacheState::default(),
        display,
        CollapseState::default(),
        ExpandableState::default(),
        GitStatsState::default(),
        VimState::default(),
        SelectionState::default(),
        DoubleClickState::default(),
    )
    .with_has_pane(false)
    .show_toc(false)
    .show_scrollbar(false)
    .show_statusline(false)
}

/// Builds a markdown widget that shows source line numbers in the gutter.
fn build_widget_with_source_gutter(markdown: &str) -> MarkdownWidget<'static> {
    let mut display = DisplaySettings::default();
    display.set_show_document_line_numbers(true);
    build_widget_with_display(markdown, display)
}

/// Renders the full terminal buffer into snapshot-friendly text.
fn render_widget(widget: &mut MarkdownWidget<'static>, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("create test terminal");
    terminal
        .draw(|frame| frame.render_widget(widget, frame.area()))
        .expect("draw markdown comment popup");

    let buffer = terminal.backend().buffer();
    let mut output = String::new();
    for y in 0..buffer.area.height {
        let mut line = String::new();
        for x in 0..buffer.area.width {
            let symbol = buffer[(x, y)].symbol();
            line.push(symbol.chars().next().unwrap_or(' '));
        }
        output.push_str(&format!("{y:02}|{line}|\n"));
    }
    output
}

#[test]
fn comment_popup_opens_and_renders_after_hotkey() {
    let mut widget = build_widget("# Title\n\nBody line");

    let event = widget.handle_key(key_event(KeyCode::Char('c'), KeyModifiers::NONE));
    assert!(
        matches!(
            event,
            MarkdownEvent::CommentPopupToggled {
                active: true,
                line: 1
            }
        ),
        "comment hotkey should open popup and request redraw: {event:?}"
    );

    let snapshot = render_widget(&mut widget, 56, 10);
    assert!(
        snapshot.contains("Comment line 1"),
        "popup title was not rendered:\n{snapshot}"
    );
    assert!(
        snapshot.contains("█"),
        "popup cursor was not rendered:\n{snapshot}"
    );
    assert_snapshot!("comment_popup_opens_after_hotkey", snapshot);
}

#[test]
fn comment_popup_draft_cursor_renders_and_enter_submits() {
    let mut widget = build_widget("# Title\n\nBody line");

    widget.handle_key(key_event(KeyCode::Char('c'), KeyModifiers::NONE));
    widget.handle_key(key_event(KeyCode::Char('h'), KeyModifiers::NONE));
    widget.handle_key(key_event(KeyCode::Char('i'), KeyModifiers::NONE));

    let snapshot = render_widget(&mut widget, 56, 10);
    assert!(
        snapshot.contains("hi█"),
        "draft cursor missing:\n{snapshot}"
    );
    assert_snapshot!("comment_popup_draft_cursor", snapshot);

    let event = widget.handle_key(key_event(KeyCode::Enter, KeyModifiers::NONE));
    assert!(
        matches!(
            event,
            MarkdownEvent::CommentSubmitted {
                line: 1,
                ref comment_text,
                ..
            } if comment_text == "hi"
        ),
        "Enter should submit comment: {event:?}"
    );
}

#[test]
fn comment_marker_renders_and_popup_reopens_existing_comment() {
    let mut widget = build_widget("# Title\n\nBody line");
    widget.set_line_comments(vec![MarkdownLineComment {
        line: 1,
        line_hash: ratkit::widgets::markdown_preview::markdown_line_content_hash("# Title"),
        line_text: "# Title".to_string(),
        comment_count: 1,
        comment_text: Some("saved comment".to_string()),
    }]);

    let marker_snapshot = render_widget(&mut widget, 56, 10);
    assert!(
        marker_snapshot.contains("●"),
        "comment marker was not rendered:\n{marker_snapshot}"
    );
    assert_snapshot!("comment_marker_renders", marker_snapshot);

    let event = widget.handle_key(key_event(KeyCode::Char('c'), KeyModifiers::NONE));
    assert!(
        matches!(
            event,
            MarkdownEvent::CommentPopupToggled {
                active: true,
                line: 1
            }
        ),
        "comment hotkey should reopen saved comment: {event:?}"
    );
    let popup_snapshot = render_widget(&mut widget, 56, 10);
    assert!(
        popup_snapshot.contains("saved comment█"),
        "saved comment was not restored in popup:\n{popup_snapshot}"
    );
    assert_snapshot!("comment_popup_reopens_saved_comment", popup_snapshot);
}

#[test]
fn comment_popup_title_matches_visible_source_line_gutter() {
    let markdown = "# Title\nfirst paragraph source line two\nsecond paragraph source line three\n\nAfter paragraph";
    let mut widget = build_widget_with_source_gutter(markdown);

    render_widget(&mut widget, 70, 8);
    for _ in 0..5 {
        widget.handle_key(key_event(KeyCode::Down, KeyModifiers::NONE));
    }

    let focused_snapshot = render_widget(&mut widget, 70, 8);
    assert!(
        focused_snapshot.contains("  5 │ After paragraph"),
        "visible gutter should use the same source line as comments:\n{focused_snapshot}"
    );

    let event = widget.handle_key(key_event(KeyCode::Char('c'), KeyModifiers::NONE));
    assert!(
        matches!(
            event,
            MarkdownEvent::CommentPopupToggled {
                active: true,
                line: 5
            }
        ),
        "comment hotkey should target the visible source line: {event:?}"
    );
    let popup_snapshot = render_widget(&mut widget, 70, 8);
    assert!(
        popup_snapshot.contains("Comment line 5"),
        "popup title should match the visible gutter source line:\n{popup_snapshot}"
    );
    assert_snapshot!("comment_popup_matches_source_gutter", popup_snapshot);
}

#[test]
fn comment_marker_stays_visible_when_scrollbar_renders() {
    let long_markdown = (1..=30)
        .map(|line| format!("line {line}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut widget = build_widget(&long_markdown).show_scrollbar(true);
    widget.set_line_comments(vec![MarkdownLineComment {
        line: 1,
        line_hash: ratkit::widgets::markdown_preview::markdown_line_content_hash("line 1"),
        line_text: "line 1".to_string(),
        comment_count: 1,
        comment_text: Some("saved comment".to_string()),
    }]);

    let marker_snapshot = render_widget(&mut widget, 40, 6);
    assert!(
        marker_snapshot.contains("●"),
        "scrollbar should not overwrite the comment marker:\n{marker_snapshot}"
    );
    assert_snapshot!("comment_marker_with_scrollbar", marker_snapshot);
}
