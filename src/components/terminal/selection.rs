//! Terminal text selection

use crate::types::terminal::TerminalGrid;

/// Selection state
#[derive(Clone, Default)]
pub struct Selection {
    /// Start position (row, col)
    pub start: Option<(u16, u16)>,

    /// End position (row, col)
    pub end: Option<(u16, u16)>,

    /// Whether selection is active (mouse button held)
    pub active: bool,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a new selection at the given position
    pub fn start_at(&mut self, row: u16, col: u16) {
        self.start = Some((row, col));
        self.end = Some((row, col));
        self.active = true;
    }

    /// Update selection end position
    pub fn update_to(&mut self, row: u16, col: u16) {
        if self.active {
            self.end = Some((row, col));
        }
    }

    /// Complete the selection
    pub fn complete(&mut self) {
        self.active = false;
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.start = None;
        self.end = None;
        self.active = false;
    }

    /// Check if a position is within the selection
    pub fn contains(&self, row: u16, col: u16) -> bool {
        let (start, end) = match (self.start, self.end) {
            (Some(s), Some(e)) => (s, e),
            _ => return false,
        };

        // Normalize start and end (ensure start <= end)
        let (start, end) = if start.0 < end.0 || (start.0 == end.0 && start.1 <= end.1) {
            (start, end)
        } else {
            (end, start)
        };

        // Check if position is in range
        if row < start.0 || row > end.0 {
            return false;
        }

        if row == start.0 && row == end.0 {
            // Single line selection
            col >= start.1 && col <= end.1
        } else if row == start.0 {
            // First line of multi-line selection
            col >= start.1
        } else if row == end.0 {
            // Last line of multi-line selection
            col <= end.1
        } else {
            // Middle lines are fully selected
            true
        }
    }

    /// Get selected text from the grid
    pub fn get_text(&self, grid: &TerminalGrid) -> String {
        let (start, end) = match (self.start, self.end) {
            (Some(s), Some(e)) => (s, e),
            _ => return String::new(),
        };

        // Normalize start and end
        let (start, end) = if start.0 < end.0 || (start.0 == end.0 && start.1 <= end.1) {
            (start, end)
        } else {
            (end, start)
        };

        let mut result = String::new();

        for row in start.0..=end.0 {
            let row_start = if row == start.0 { start.1 } else { 0 };
            let row_end = if row == end.0 {
                end.1
            } else {
                grid.cols as u16 - 1
            };

            for col in row_start..=row_end {
                if let Some(cell) = grid
                    .cells
                    .iter()
                    .find(|c| c.row == row && c.col == col)
                {
                    result.push_str(&cell.content);
                } else {
                    result.push(' ');
                }
            }

            // Add newline between rows (but not at the end)
            if row < end.0 {
                result.push('\n');
            }
        }

        // Trim trailing spaces from each line
        result
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Check if there is an active selection
    pub fn has_selection(&self) -> bool {
        self.start.is_some() && self.end.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_contains() {
        let mut sel = Selection::new();
        sel.start = Some((1, 5));
        sel.end = Some((3, 10));

        // Should contain
        assert!(sel.contains(1, 5));
        assert!(sel.contains(1, 10));
        assert!(sel.contains(2, 0));
        assert!(sel.contains(2, 50));
        assert!(sel.contains(3, 0));
        assert!(sel.contains(3, 10));

        // Should not contain
        assert!(!sel.contains(0, 5));
        assert!(!sel.contains(1, 4));
        assert!(!sel.contains(3, 11));
        assert!(!sel.contains(4, 5));
    }

    #[test]
    fn test_selection_single_line() {
        let mut sel = Selection::new();
        sel.start = Some((2, 5));
        sel.end = Some((2, 10));

        assert!(sel.contains(2, 5));
        assert!(sel.contains(2, 7));
        assert!(sel.contains(2, 10));
        assert!(!sel.contains(2, 4));
        assert!(!sel.contains(2, 11));
    }

    #[test]
    fn test_selection_reversed() {
        let mut sel = Selection::new();
        // Selection from bottom-right to top-left
        sel.start = Some((3, 10));
        sel.end = Some((1, 5));

        assert!(sel.contains(1, 5));
        assert!(sel.contains(2, 0));
        assert!(sel.contains(3, 10));
    }
}
