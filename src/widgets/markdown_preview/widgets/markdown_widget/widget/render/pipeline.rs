use crate::primitives::pane::Pane;
use crate::widgets::document_viewer::{DocumentViewerWidget, ScrollState as ViewerScrollState};
use crate::widgets::markdown_preview::widgets::markdown_widget::extensions::scrollbar::CustomScrollbar;
use crate::widgets::markdown_preview::widgets::markdown_widget::extensions::selection::should_render_line;
use crate::widgets::markdown_preview::widgets::markdown_widget::extensions::toc::Toc;
use crate::widgets::markdown_preview::widgets::markdown_widget::foundation::elements::{
    render_with_options, RenderOptions,
};
use crate::widgets::markdown_preview::widgets::markdown_widget::foundation::helpers::hash_content;
use crate::widgets::markdown_preview::widgets::markdown_widget::markdown_document_adapter::markdown_viewport_lines_to_document_with_source_lines;
use crate::widgets::markdown_preview::widgets::markdown_widget::markdown_viewer_state_adapter::markdown_display_to_viewer_display;
use crate::widgets::markdown_preview::widgets::markdown_widget::state::{ParsedCache, RenderCache};
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::features::filter::element_to_plain_text_for_filter;
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::features::selection::apply_selection_highlighting;
use crate::widgets::markdown_preview::widgets::markdown_widget::widget::{
    MarkdownWidget, CURRENT_LINE_BG, CURRENT_LINE_DRAG_BG,
};
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Widget};

impl<'a> Widget for &mut MarkdownWidget<'a> {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let (area, _pane_footer_area) = if self.has_pane {
            let title = self
                .pane_title
                .clone()
                .unwrap_or_else(|| "Markdown".to_string());
            let pane = self.pane.take().unwrap_or_else(|| {
                let mut p = Pane::new(title);
                if let Some(color) = self.pane_color {
                    p = p.border_style(ratatui::style::Style::default().fg(color));
                }
                p
            });

            let mut block = Block::default()
                .borders(Borders::ALL)
                .border_type(pane.border_type)
                .border_style(pane.border_style)
                .title(pane.title);

            if let Some(icon) = &pane.icon {
                let title = format!(" {} ", icon);
                block = block.title(Line::from(vec![Span::styled(title, pane.title_style)]));
            }

            if let Some(ref footer) = pane.text_footer {
                block = block.title_bottom(footer.clone().style(pane.footer_style));
            }

            let inner = block.inner(area);
            block.render(area, buf);

            let (inner, pane_footer) = if pane.footer_height > 0 {
                let chunks = ratatui::layout::Layout::default()
                    .direction(ratatui::layout::Direction::Vertical)
                    .constraints([
                        ratatui::layout::Constraint::Min(0),
                        ratatui::layout::Constraint::Length(pane.footer_height),
                    ])
                    .split(inner);
                (chunks[0], Some(chunks[1]))
            } else {
                (inner, None)
            };

            let padded = Rect {
                x: inner.x + pane.padding.3,
                y: inner.y + pane.padding.0,
                width: inner.width.saturating_sub(pane.padding.1 + pane.padding.3),
                height: inner.height.saturating_sub(pane.padding.0 + pane.padding.2),
            };

            if let Some(footer_area) = pane_footer {
                if let Some(ref footer) = pane.text_footer {
                    footer.render(footer_area, buf);
                }
            }

            (padded, None::<Rect>)
        } else {
            (area, None::<Rect>)
        };

        let (main_area, statusline_area) = if self.show_statusline && area.height > 1 {
            (
                Rect {
                    height: area.height.saturating_sub(1),
                    ..area
                },
                Some(Rect {
                    y: area.y + area.height.saturating_sub(1),
                    height: 1,
                    ..area
                }),
            )
        } else {
            (area, None)
        };

        let content_area = main_area;
        let overlay_area = self.calculate_toc_area(main_area);

        self.scroll.update_viewport(content_area);

        let line_num_width = if self.display.show_document_line_numbers() {
            6
        } else {
            0
        };

        let width = (content_area.width as usize).saturating_sub(line_num_width);
        let content_hash = hash_content(&self.content);
        let show_line_numbers = self.display.show_line_numbers;
        let theme = self.display.code_block_theme;

        let app_theme_hash = self
            .app_theme
            .as_ref()
            .map(|t| {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};
                let mut hasher = DefaultHasher::new();
                format!(
                    "{:?}{:?}{:?}{:?}{:?}",
                    t.primary, t.text, t.background, t.markdown.heading, t.markdown.code
                )
                .hash(&mut hasher);
                hasher.finish()
            })
            .unwrap_or(0);

        let show_heading_collapse = self.display.show_heading_collapse;
        let render_cache_valid = !self.filter_mode
            && self
                .cache
                .render
                .as_ref()
                .map(|c| {
                    c.content_hash == content_hash
                        && c.width == width
                        && c.show_line_numbers == show_line_numbers
                        && c.theme == theme
                        && c.app_theme_hash == app_theme_hash
                        && c.show_heading_collapse == show_heading_collapse
                })
                .unwrap_or(false);

        if !render_cache_valid {
            let parsed_cache_valid = self
                .cache
                .parsed
                .as_ref()
                .map(|c| c.content_hash == content_hash)
                .unwrap_or(false);

            let elements = if parsed_cache_valid {
                self.cache
                    .parsed
                    .as_ref()
                    .expect("parsed cache present")
                    .elements
                    .clone()
            } else {
                let parsed = self.parse_elements();
                self.cache.parsed = Some(ParsedCache {
                    content_hash,
                    elements: parsed.clone(),
                });
                parsed
            };

            let render_options = RenderOptions {
                show_line_numbers,
                theme,
                app_theme: self.app_theme.as_ref(),
                show_heading_collapse: self.display.show_heading_collapse,
            };

            let filter_lower = self
                .filter_mode
                .then(|| self.filter.as_deref().unwrap_or("").to_lowercase());

            let mut lines: Vec<Line<'static>> = Vec::new();
            let mut boundaries: Vec<(usize, usize)> = Vec::new();
            let mut source_lines: Vec<usize> = Vec::new();

            for (idx, element) in elements.iter().enumerate() {
                if !should_render_line(element, idx, &self.collapse) {
                    continue;
                }

                if let Some(ref filter) = filter_lower {
                    let text = element_to_plain_text_for_filter(&element.kind).to_lowercase();
                    if !text.contains(filter) {
                        continue;
                    }
                }

                let start_idx = lines.len();
                let rendered = render_with_options(element, width, render_options);
                let line_count = rendered.len();
                let source_line = element.source_line.max(1);
                lines.extend(rendered);
                source_lines.extend(std::iter::repeat(source_line).take(line_count));
                boundaries.push((start_idx, line_count));
            }

            self.cache.render = Some(RenderCache {
                content_hash,
                width,
                show_line_numbers,
                theme,
                app_theme_hash,
                show_heading_collapse,
                lines,
                line_boundaries: boundaries,
                line_source_lines: source_lines,
            });
        }

        let render_cache = self.cache.render.as_ref().expect("render cache present");
        self.scroll.update_total_lines(render_cache.lines.len());
        if !render_cache_valid || self.rendered_lines.len() != render_cache.lines.len() {
            self.rendered_lines = render_cache.lines.clone();
        }

        let current_visual_line = self.scroll.current_line.saturating_sub(1);
        let visible_range = self.scroll.visible_range(content_area.height as usize);
        let decorated_lines = self
            .selection_active
            .then(|| apply_selection_highlighting(render_cache.lines.clone(), &self.selection, 0));

        let final_lines: Vec<Line<'static>> = if self.display.show_document_line_numbers() {
            let theme_colors = self.display.code_block_theme.colors();
            let line_num_style = Style::default()
                .fg(theme_colors.line_number)
                .bg(theme_colors.background);
            let border_style = Style::default()
                .fg(theme_colors.border)
                .bg(theme_colors.background);
            visible_range
                .clone()
                .filter_map(|visual_idx| {
                    let line = decorated_lines
                        .as_ref()
                        .and_then(|lines| lines.get(visual_idx))
                        .or_else(|| render_cache.lines.get(visual_idx))?
                        .clone();
                    let (logical_num, is_first) =
                        visual_to_logical_line(&render_cache.line_boundaries, visual_idx)
                            .unwrap_or((visual_idx + 1, true));
                    Some(decorate_markdown_line(
                        line,
                        visual_idx,
                        current_visual_line,
                        Some((logical_num, is_first, line_num_style, border_style)),
                        self.selection_active,
                        content_area.width as usize,
                        line_num_width,
                    ))
                })
                .collect()
        } else {
            visible_range
                .clone()
                .filter_map(|visual_idx| {
                    let line = decorated_lines
                        .as_ref()
                        .and_then(|lines| lines.get(visual_idx))
                        .or_else(|| render_cache.lines.get(visual_idx))?
                        .clone();
                    Some(decorate_markdown_line(
                        line,
                        visual_idx,
                        current_visual_line,
                        None,
                        self.selection_active,
                        content_area.width as usize,
                        line_num_width,
                    ))
                })
                .collect()
        };

        let visible_source_lines = visible_range
            .clone()
            .map(|visual_idx| {
                render_cache
                    .line_source_lines
                    .get(visual_idx)
                    .copied()
                    .unwrap_or_else(|| visual_idx.saturating_add(1))
            })
            .collect::<Vec<_>>();
        let markdown_document = markdown_viewport_lines_to_document_with_source_lines(
            final_lines,
            visible_source_lines,
        );
        let mut viewer_scroll = ViewerScrollState::default();
        viewer_scroll.viewport_height = content_area.height as usize;
        viewer_scroll.update_total_lines(markdown_document.line_count().max(1));
        viewer_scroll.set_current_line(
            current_visual_line
                .saturating_sub(visible_range.start)
                .saturating_add(1),
        );
        viewer_scroll.set_offset(0);
        let mut viewer_display = markdown_display_to_viewer_display(&self.display);
        viewer_display.show_line_numbers = false;
        viewer_display.highlight_current_line = false;
        let viewer = DocumentViewerWidget::new(markdown_document, viewer_display);
        viewer.render(content_area, buf, &viewer_scroll);

        if let Some(ov_area) = overlay_area {
            self.sync_toc_interaction_state();
            let final_state = self.toc_state.as_ref().expect("toc state present");
            let toc = Toc::new(final_state)
                .expanded(self.toc_hovered)
                .config(self.toc_config.clone());

            toc.render(ov_area, buf);
        }

        if let Some(sl_area) = statusline_area {
            self.render_statusline(sl_area, buf);
        }

        if self.show_scrollbar && self.scroll.total_lines > content_area.height as usize {
            let scrollbar_width = self.scrollbar_config.width;
            let scrollbar_area = Rect {
                x: content_area.x + content_area.width.saturating_sub(scrollbar_width),
                y: content_area.y,
                width: scrollbar_width,
                height: content_area.height,
            };

            let scrollbar = CustomScrollbar::new(&self.scroll)
                .config(self.scrollbar_config.clone())
                .show_percentage(false);

            scrollbar.render(scrollbar_area, buf);
        }

        self.inner_area = Some(content_area);
    }
}

/// Maps a rendered visual line index to its logical markdown line number.
fn visual_to_logical_line(
    line_boundaries: &[(usize, usize)],
    visual_idx: usize,
) -> Option<(usize, bool)> {
    let candidate = line_boundaries
        .partition_point(|(start_idx, _count)| *start_idx <= visual_idx)
        .checked_sub(1)?;
    let (start_idx, count) = line_boundaries.get(candidate).copied()?;
    (visual_idx < start_idx.saturating_add(count))
        .then_some((candidate.saturating_add(1), visual_idx == start_idx))
}

/// Adds optional document line numbers and current-line highlighting to one visible line.
#[allow(clippy::type_complexity)]
fn decorate_markdown_line(
    mut line: Line<'static>,
    visual_idx: usize,
    current_visual_line: usize,
    line_number: Option<(usize, bool, Style, Style)>,
    selection_active: bool,
    content_width: usize,
    line_num_width: usize,
) -> Line<'static> {
    let highlight_bg = if selection_active {
        CURRENT_LINE_DRAG_BG
    } else {
        CURRENT_LINE_BG
    };
    let is_current = visual_idx == current_visual_line;
    let mut new_spans = Vec::new();
    if let Some((logical_num, is_first, line_num_style, border_style)) = line_number {
        let num_str = if is_first {
            format!("{:>3} ", logical_num)
        } else {
            "    ".to_string()
        };
        new_spans.push(Span::styled(num_str, line_num_style));
        new_spans.push(Span::styled("│ ".to_string(), border_style));
    }

    if !is_current {
        new_spans.append(&mut line.spans);
        return Line::from(new_spans);
    }

    let mut rendered_width = 0usize;
    for span in line.spans.drain(..) {
        rendered_width += span.content.chars().count();
        if span.content.contains('▋') {
            new_spans.push(span);
        } else {
            new_spans.push(Span::styled(span.content, span.style.bg(highlight_bg)));
        }
    }
    let total_width = line_num_width.saturating_add(rendered_width);
    if total_width < content_width {
        new_spans.push(Span::styled(
            " ".repeat(content_width - total_width),
            Style::default().bg(highlight_bg),
        ));
    }
    Line::from(new_spans)
}

impl<'a> Widget for MarkdownWidget<'a> {
    fn render(mut self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        <&mut MarkdownWidget<'a> as Widget>::render(&mut self, area, buf);
    }
}
