//! Syntax highlighting for code-widget lines.

use ratatui::text::Span;

use crate::widgets::{
    code_widget::foundation::CodeLanguage,
    document_viewer::{DocumentLine, DocumentLineKind},
};

/// Colour output mode for syntax highlighting.
///
/// - `TrueColor` — 24-bit RGB via `Color::Rgb(r,g,b)` (`\x1b[38;2;r;g;bm`).
///   Best for modern terminals with `$COLORTERM=truecolor` and GUI renderers.
/// - `Ansi256` — nearest ANSI 256 palette entry via `Color::Indexed(n)`.
///   Reliable on Apple_Terminal and older terminals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HighlightColorMode {
    /// 24-bit true-colour. Requires `$COLORTERM=truecolor` or a GUI renderer.
    TrueColor,
    /// ANSI 256-colour palette. Safe default for Apple Terminal / tmux.
    #[default]
    Ansi256,
}

/// Highlights source code into normalized document lines (ANSI 256 mode).
pub fn highlight_code_lines(content: &str, language: &CodeLanguage) -> Vec<DocumentLine> {
    highlight_code_lines_with_mode(content, language, HighlightColorMode::Ansi256)
}

/// Highlights source code with explicit colour mode.
pub fn highlight_code_lines_with_mode(
    content: &str,
    language: &CodeLanguage,
    mode: HighlightColorMode,
) -> Vec<DocumentLine> {
    let plain_lines = plain_document_lines(content);
    let Some(token) = language.syntect_token() else {
        return plain_lines;
    };

    highlight_with_syntect(content, token, mode).unwrap_or(plain_lines)
}

/// Converts source content into plain document lines.
fn plain_document_lines(content: &str) -> Vec<DocumentLine> {
    let mut lines: Vec<DocumentLine> = content
        .lines()
        .enumerate()
        .map(|(index, line)| {
            let kind = if line.is_empty() {
                DocumentLineKind::Empty
            } else {
                DocumentLineKind::Code
            };
            DocumentLine::plain(index + 1, line.to_string(), kind)
        })
        .collect();

    if lines.is_empty() {
        lines.push(DocumentLine::plain(1, "", DocumentLineKind::Empty));
    }
    lines
}

/// Uses syntect to highlight source when a syntax exists.
fn highlight_with_syntect(content: &str, token: &str, mode: HighlightColorMode) -> Option<Vec<DocumentLine>> {
    use std::sync::OnceLock;
    static SYNTAX_SET: OnceLock<syntect::parsing::SyntaxSet> = OnceLock::new();
    static THEME: OnceLock<syntect::highlighting::Theme> = OnceLock::new();

    let syntax_set = SYNTAX_SET.get_or_init(|| {
        let ss = syntect::parsing::SyntaxSet::load_defaults_newlines();
        tracing::debug!("syntect: loaded {} syntaxes", ss.syntaxes().len());
        ss
    });
    let theme = THEME.get_or_init(|| {
        let ts = syntect::highlighting::ThemeSet::load_defaults();
        let available: Vec<&str> = ts.themes.keys().map(|s| s.as_str()).collect();
        tracing::debug!("syntect: {} themes available: {:?}", available.len(), available);
        // Only use dark themes (not light themes like InspiredGitHub)
        let dark_names = ["base16-eighties.dark", "base16-ocean.dark", "base16-mocha.dark", "Solarized (dark)"];
        let theme = dark_names.iter()
            .find_map(|name| ts.themes.get(*name))
            .or_else(|| ts.themes.values().find(|t| {
                // Skip themes that look light (light backgrounds produce dark text)
                t.settings.background.map_or(true, |bg| {
                    // Simple heuristic: if background is bright, skip it
                    let brightness = bg.r as u32 + bg.g as u32 + bg.b as u32;
                    brightness < 400 // dark backgrounds have low RGB sum
                })
            }))
            .or_else(|| ts.themes.values().next())
            .cloned()
            .unwrap_or_else(|| {
                tracing::warn!("syntect: no theme found, using fallback");
                syntect::highlighting::Theme::default()
            });
        tracing::debug!("syntect: using theme '{}'", theme.name.as_deref().unwrap_or("unnamed"));
        theme
    });

    let syntax = syntax_set
        .find_syntax_by_token(token)
        .or_else(|| syntax_set.find_syntax_by_extension(token));
    let Some(syntax) = syntax else {
        tracing::debug!("syntect: no syntax found for token='{token}', falling back to plain text");
        return None;
    };
    tracing::debug!("syntect: using syntax '{}' for token='{token}'", syntax.name);
    let mut highlighter = syntect::easy::HighlightLines::new(syntax, theme);
    let mut lines = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let spans = match highlighter.highlight_line(line, &syntax_set) {
            Ok(regions) => regions
                .into_iter()
                .map(|(style, text)| {
                    let style = syntax_style_without_background(style, mode);
                    Span::styled(text.to_string(), style)
                })
                .collect(),
            Err(_) => vec![Span::raw(line.to_string())],
        };
        let kind = if line.is_empty() {
            DocumentLineKind::Empty
        } else {
            DocumentLineKind::Code
        };
        lines.push(DocumentLine::new(index + 1, spans, kind));
    }

    if lines.is_empty() {
        lines.push(DocumentLine::plain(1, "", DocumentLineKind::Empty));
    }
    Some(lines)
}

/// Translates syntect highlighting style to ratatui style, dropping background.
///
/// Pipeline: syntect RGB → saturation boost (base16 themes are too muted) →
/// ANSI 256 mapping (Apple Terminal handles indexed colours more reliably
/// than 24-bit true-colour escape sequences).
fn syntax_style_without_background(
    style: syntect::highlighting::Style,
    mode: HighlightColorMode,
) -> ratatui::style::Style {
    use ratatui::style::{Color, Modifier, Style};
    let sc = style.foreground;
    let fg = if sc.a == 0 {
        Color::White
    } else {
        // 1. Boost saturation away from mid-gray (base16 themes are too muted)
        let (r, g, b) = boost_saturation(sc.r, sc.g, sc.b, 1.8);
        match mode {
            HighlightColorMode::TrueColor => Color::Rgb(r, g, b),
            HighlightColorMode::Ansi256 => Color::Indexed(rgb_to_ansi256(r, g, b)),
        }
    };
    let mut s = Style::default().fg(fg);
    if style.font_style.contains(syntect::highlighting::FontStyle::BOLD) {
        s = s.add_modifier(Modifier::BOLD);
    }
    if style.font_style.contains(syntect::highlighting::FontStyle::ITALIC) {
        s = s.add_modifier(Modifier::ITALIC);
    }
    if style.font_style.contains(syntect::highlighting::FontStyle::UNDERLINE) {
        s = s.add_modifier(Modifier::UNDERLINED);
    }
    s
}

/// Boost colour saturation by expanding each channel away from mid-gray (128).
fn boost_saturation(r: u8, g: u8, b: u8, factor: f32) -> (u8, u8, u8) {
    let boost = |c: u8| -> u8 {
        let delta = c as f32 - 128.0;
        (128.0 + delta * factor).clamp(0.0, 255.0) as u8
    };
    (boost(r), boost(g), boost(b))
}

/// Map 24-bit RGB to the nearest ANSI 256-colour palette entry.
///
/// Palette structure:
///   0–7:   Standard ANSI colours
///   8–15:  Bright ANSI colours
///  16–231: 6×6×6 RGB cube (216 colours)
/// 232–255: Greyscale ramp (24 steps)
fn rgb_to_ansi256(r: u8, g: u8, b: u8) -> u8 {
    if r == g && g == b {
        if r < 8 { return 16; }
        if r > 248 { return 231; }
        return 232 + ((r as u16 - 8) * 24 / 247) as u8;
    }
    let r6 = (r as u16 * 6 / 256) as u8;
    let g6 = (g as u16 * 6 / 256) as u8;
    let b6 = (b as u16 * 6 / 256) as u8;
    16 + 36 * r6 + 6 * g6 + b6
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_plain_text_for_unknown_language() {
        let lines = highlight_code_lines("one\ntwo", &CodeLanguage::Unknown);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[1].plain_text(), "two");
    }
}
