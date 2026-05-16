//! Stateful ratatui widget for displaying parsed code diffs.

use std::collections::HashMap;

use super::foundation::diff_config::DiffConfig;
use super::parse::parse_unified_diff;
use super::types::{DiffFile, DiffHunk};
use ratatui::layout::Rect;
use ratatui::widgets::Widget;

/// A reusable code diff widget with host-owned input data.
#[derive(Debug, Clone, Default)]
pub struct CodeDiff {
    /// Currently selected or displayed file path.
    pub file_path: Option<String>,
    /// Hunks for the currently selected file.
    pub hunks: Vec<DiffHunk>,
    /// Rendering configuration.
    pub config: DiffConfig,
    /// First visible row offset for host-managed scrolling.
    pub scroll_offset: usize,
    /// Parsed hunks keyed by file path for backward-compatible access.
    pub file_diffs: HashMap<String, Vec<DiffHunk>>,
    /// Parsed file entries from the latest unified diff input.
    pub files: Vec<DiffFile>,
}

impl CodeDiff {
    /// Creates an empty code diff widget.
    pub fn new() -> Self {
        Self {
            file_path: None,
            hunks: Vec::new(),
            config: DiffConfig::new(),
            scroll_offset: 0,
            file_diffs: HashMap::new(),
            files: Vec::new(),
        }
    }

    /// Sets the display file path.
    pub fn with_file_path(mut self, path: &str) -> Self {
        self.file_path = Some(path.to_string());
        self
    }

    /// Sets the rendering configuration.
    pub fn with_config(mut self, config: DiffConfig) -> Self {
        self.config = config;
        self
    }

    /// Appends a hunk to the currently selected file hunks.
    pub fn add_hunk(&mut self, hunk: DiffHunk) {
        self.hunks.push(hunk);
    }

    /// Returns hunks for the currently selected file.
    pub fn hunks(&self) -> &[DiffHunk] {
        &self.hunks
    }

    /// Returns mutable hunks for the currently selected file.
    pub fn hunks_mut(&mut self) -> &mut [DiffHunk] {
        &mut self.hunks
    }

    /// Builds a widget by parsing unified diff text.
    pub fn from_unified_diff(diff: &str) -> Self {
        let files = parse_unified_diff(diff);
        Self::from_files(files)
    }

    /// Builds a widget from parsed diff file entries.
    pub fn from_files(files: Vec<DiffFile>) -> Self {
        let mut widget = Self::new();
        widget.files = files;

        if let Some(first_file) = widget.files.first() {
            widget.file_path = Some(first_file.path.clone());
            widget.hunks = first_file.hunks.clone();
        }

        widget.file_diffs = widget
            .files
            .iter()
            .map(|file| (file.path.clone(), file.hunks.clone()))
            .collect();
        widget
    }
}

impl Widget for CodeDiff {
    fn render(self, area: Rect, buf: &mut ratatui::buffer::Buffer) {
        let content = self
            .file_path
            .as_deref()
            .map(|path| format!("Diff: {path}"))
            .unwrap_or_else(|| "Diff: (no file)".to_string());
        let widget = ratatui::widgets::Paragraph::new(content);
        widget.render(area, buf);
    }
}
