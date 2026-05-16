/// Selects the visual diff layout used by the widget.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffStyle {
    /// Render old and new content in separate columns.
    #[default]
    SideBySide,
    /// Render content in one unified column.
    Unified,
}
