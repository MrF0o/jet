/// Scroll handling module
use crate::App;
use ratatui::prelude::Rect;

impl App {
    /// Handle mouse scroll with proper bounds check on editor area bounds
    pub fn handle_mouse_scroll(&mut self, delta: i16, editor_area: Rect) {
        // Update scroll offset
        let (scroll_row, _scroll_col) = self.scroll_offset;

        let new_scroll_row = if delta > 0 {
            // Scrolling down - increase row offset
            scroll_row + delta as usize
        } else {
            // Scrolling up - decrease row offset
            scroll_row.saturating_sub((-delta) as usize)
        };

        // Limit scroll to buffer content with proper bounds checking
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let editor_height = editor_area.height as usize;

            // Allow scrolling past the end of buffer to see final lines comfortably
            // Add half the editor height as extra scrollable space
            let max_scroll = if buffer.content.len() > editor_height {
                buffer.content.len() + (editor_height / 2) - editor_height
            } else {
                0
            };

            self.scroll_offset.0 = new_scroll_row.min(max_scroll);
        }

        // Manual scroll shouldn't move the cursor - we're just changing the view

        // Notify cursor manager about activity
        self.cursor_manager.notify_activity_for_active();
    }

    /// Handle vertical scrolling with key input (Page Up/Down)
    pub fn handle_key_scroll(&mut self, lines: i16, editor_area: Rect) {
        // For page up/down, use the actual editor area height as page size
        // Otherwise use the lines parameter (like for mouse wheel scroll)
        let adjusted_lines = if lines.abs() >= 8 {
            // This is likely a page up/down operation, use terminal height
            let page_size = editor_area.height as i16;
            if lines > 0 {
                page_size
            } else {
                -page_size
            }
        } else {
            lines
        };

        self.handle_mouse_scroll(adjusted_lines, editor_area);
    }

    /// Ensure cursor is visible within the editor area (scroll if needed)
    pub fn ensure_cursor_visible_with_area(&mut self, area: Rect) {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let (row, col) = buffer.cursor_pos;
            let (scroll_row, scroll_col) = self.scroll_offset;

            // Define scroll margins - keep cursor at least 3 lines from edges when possible
            let scroll_margin = 3;
            let visible_rows = area.height as usize;

            // Adjust vertical scroll with margin consideration
            if row < scroll_row + scroll_margin {
                // Cursor is too close to the top, scroll up
                self.scroll_offset.0 = row.saturating_sub(scroll_margin);
            } else if row >= scroll_row + visible_rows - scroll_margin {
                // Cursor is too close to the bottom, scroll down
                let new_scroll = row.saturating_sub(visible_rows.saturating_sub(scroll_margin + 1));
                self.scroll_offset.0 = new_scroll;
            }

            // Adjust horizontal scroll if needed (account for line numbers)
            let line_number_width = if self.get_line_numbers_setting() {
                buffer.line_number_width()
            } else {
                0
            };
            let visible_cols = area.width as usize - line_number_width;

            if col < scroll_col {
                self.scroll_offset.1 = col;
            } else if col >= scroll_col + visible_cols {
                self.scroll_offset.1 = col.saturating_sub(visible_cols) + 1;
            }
        }
    }

    /// Get the maximum scroll position for the current buffer
    pub fn get_max_scroll_row(&self, editor_area: Rect) -> usize {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let editor_height = editor_area.height as usize;
            if buffer.content.len() > editor_height {
                buffer.content.len() - editor_height
            } else {
                0
            }
        } else {
            0
        }
    }

    /// Get the maximum horizontal scroll position for the current buffer
    pub fn get_max_scroll_col(&self, editor_area: Rect) -> usize {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let line_number_width = if self.get_line_numbers_setting() {
                buffer.line_number_width()
            } else {
                0
            };
            let visible_cols = editor_area.width as usize - line_number_width;

            // Find the longest line in the buffer
            let max_line_length = buffer
                .content
                .iter()
                .map(|line| line.len())
                .max()
                .unwrap_or(0);

            max_line_length.saturating_sub(visible_cols)
        } else {
            0
        }
    }
}
