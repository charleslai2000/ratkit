//! Backward-compatible foundation exports for the code diff widget.

/// Configuration for code diff rendering.
pub mod diff_config {
    use ratatui::style::Color;

    /// Controls colors, gutters, and context settings for diff rendering.
    #[derive(Debug, Clone, Default)]
    pub struct DiffConfig {
        /// Whether line numbers are displayed.
        pub show_line_numbers: bool,
        /// Foreground color for added lines.
        pub added_fg: Color,
        /// Background color for added lines.
        pub added_bg: Color,
        /// Foreground color for removed lines.
        pub removed_fg: Color,
        /// Background color for removed lines.
        pub removed_bg: Color,
        /// Foreground color for unchanged context.
        pub context_fg: Color,
        /// Background color for unchanged context.
        pub context_bg: Color,
        /// Foreground color for hunk headers.
        pub hunk_header_fg: Color,
        /// Background color for hunk headers.
        pub hunk_header_bg: Color,
        /// Foreground color for line-number gutters.
        pub line_number_fg: Color,
        /// Width of line-number gutters.
        pub gutter_width: usize,
        /// Number of context lines shown around changes.
        pub context_lines: usize,
    }

    impl DiffConfig {
        /// Creates the default diff rendering configuration.
        pub fn new() -> Self {
            Self {
                show_line_numbers: true,
                added_fg: Color::Black,
                added_bg: Color::LightGreen,
                removed_fg: Color::White,
                removed_bg: Color::LightRed,
                context_fg: Color::Gray,
                context_bg: Color::Reset,
                hunk_header_fg: Color::DarkGray,
                hunk_header_bg: Color::Reset,
                line_number_fg: Color::DarkGray,
                gutter_width: 4,
                context_lines: 3,
            }
        }

        /// Sets whether line numbers are displayed.
        pub fn with_show_line_numbers(mut self, show: bool) -> Self {
            self.show_line_numbers = show;
            self
        }
    }
}

/// Parsed diff line types.
pub mod diff_line {
    pub use crate::widgets::code_diff::code_diff::types::{DiffLine, DiffLineKind};
}

/// Parsed diff hunk types.
pub mod diff_hunk {
    pub use crate::widgets::code_diff::code_diff::types::DiffHunk;
}

/// Diff rendering enums.
pub mod enums {
    pub use crate::widgets::code_diff::code_diff::types::{DiffLineKind, DiffStyle};
}

/// Legacy helper functions for pure text diff generation.
pub mod helpers {
    use similar::{ChangeTag, TextDiff};

    /// Builds a pure unified text diff without performing VCS IO.
    pub fn get_git_diff(old: &str, new: &str) -> String {
        if old == new {
            return String::new();
        }

        let old_count = old.lines().count();
        let new_count = new.lines().count();
        let mut output = format!("@@ -1,{old_count} +1,{new_count} @@\n");
        let diff = TextDiff::from_lines(old, new);

        for change in diff.iter_all_changes() {
            let prefix = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            output.push_str(prefix);
            output.push_str(change.value());
            if !change.value().ends_with('\n') {
                output.push('\n');
            }
        }

        output
    }
}
