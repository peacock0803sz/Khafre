//! Terminal-related type definitions

use crate::types::color_scheme::Rgb;

/// Terminal grid representation for rendering
#[derive(Clone, Default)]
pub struct TerminalGrid {
    /// All cells in the visible area
    pub cells: Vec<CellInfo>,

    /// Cursor information
    pub cursor: CursorInfo,

    /// Number of columns
    pub cols: usize,

    /// Number of rows
    pub rows: usize,
}

/// Information about a single cell
#[derive(Clone)]
pub struct CellInfo {
    /// Row position
    pub row: u16,

    /// Column position
    pub col: u16,

    /// Character content
    pub content: String,

    /// Foreground color
    pub fg: Rgb,

    /// Background color
    pub bg: Rgb,

    /// Cell flags (bold, italic, underline, etc.)
    pub flags: CellFlags,
}

/// Cell style flags
#[derive(Clone, Copy, Default)]
pub struct CellFlags {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub inverse: bool,
    pub hidden: bool,
}

/// Cursor information
#[derive(Clone, Default)]
pub struct CursorInfo {
    /// Row position
    pub row: u16,

    /// Column position
    pub col: u16,

    /// Whether cursor is visible
    pub visible: bool,

    /// Cursor shape
    pub shape: CursorShape,
}

/// Cursor shape
#[derive(Clone, Copy, Default, PartialEq)]
pub enum CursorShape {
    #[default]
    Block,
    Underline,
    Beam,
}

/// Terminal event
#[derive(Clone, Debug)]
pub enum TerminalEvent {
    /// Terminal title changed
    Title(String),

    /// Bell
    Bell,

    /// Exit
    Exit,

    /// Clipboard request
    ClipboardStore(String),

    /// Clipboard request
    ClipboardLoad,
}
