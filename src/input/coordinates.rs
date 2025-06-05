// Coordinate conversion and screen layout management

use crate::App;
use ratatui::layout::Rect;

/// Convert screen coordinates to buffer coordinates
/// Takes into account the current editor layout, scroll offset, and line numbers
pub fn screen_to_buffer_coords(app: &App, mouse_x: u16, mouse_y: u16) -> Option<(usize, usize)> {
    // Get the actual editor area - this should be passed from the UI layer
    // For now, we'll calculate it based on the application state
    let editor_area = get_editor_area();

    // Check if click is within editor area
    if mouse_x < editor_area.x
        || mouse_x >= editor_area.x + editor_area.width
        || mouse_y < editor_area.y
        || mouse_y >= editor_area.y + editor_area.height
    {
        return None;
    }

    // Calculate relative position within editor
    let relative_x = mouse_x - editor_area.x;
    let relative_y = mouse_y - editor_area.y;

    // Account for line numbers if enabled
    let line_number_width = if app.get_line_numbers_setting() {
        if let Some(buffer) = app.buffers.get(app.active_buffer) {
            buffer.line_number_width()
        } else {
            0
        }
    } else {
        0
    };

    // Check if click is in line number area
    if relative_x < line_number_width as u16 {
        // Click is in line number area - position cursor at beginning of line
        let (scroll_row, _) = app.scroll_offset;
        let buffer_row = scroll_row + relative_y as usize;

        if let Some(buffer) = app.buffers.get(app.active_buffer) {
            if buffer_row < buffer.content.len() {
                return Some((buffer_row, 0));
            }
        }
        return None;
    }

    let text_relative_x = relative_x - line_number_width as u16;

    // Apply scroll offset
    let (scroll_row, scroll_col) = app.scroll_offset;
    let buffer_row = scroll_row + relative_y as usize;
    let buffer_col = scroll_col + text_relative_x as usize;

    // Validate coordinates against buffer content
    if let Some(buffer) = app.buffers.get(app.active_buffer) {
        if buffer_row >= buffer.content.len() {
            // Click is beyond buffer content - position at end of last line
            let last_row = buffer.content.len().saturating_sub(1);
            let last_col = buffer
                .content
                .get(last_row)
                .map(|line| line.len())
                .unwrap_or(0);
            return Some((last_row, last_col));
        }

        let line = &buffer.content[buffer_row];
        let adjusted_col = buffer_col.min(line.len());
        return Some((buffer_row, adjusted_col));
    }

    None
}

/// Get the editor area bounds
/// This should eventually be passed from the UI rendering layer
/// For now, we'll use a reasonable approximation
fn get_editor_area() -> Rect {
    // Try to get actual terminal size
    if let Ok((width, height)) = ratatui::crossterm::terminal::size() {
        Rect {
            x: 0,
            y: 0,
            width,
            height: height.saturating_sub(1), // -1 for status line
        }
    } else {
        // Fallback to default size
        Rect {
            x: 0,
            y: 0,
            width: 120,
            height: 29,
        }
    }
}

/// Calculate the editor area based on terminal size and UI layout
pub fn calculate_editor_area(terminal_area: Rect) -> Rect {
    // Account for status line at the bottom
    Rect {
        x: 0,
        y: 0,
        width: terminal_area.width,
        height: terminal_area.height.saturating_sub(1), // -1 for status line
    }
}

/// Convert buffer coordinates to screen coordinates
pub fn buffer_to_screen_coords(
    app: &App,
    buffer_row: usize,
    buffer_col: usize,
    editor_area: Rect,
) -> Option<(u16, u16)> {
    let (scroll_row, scroll_col) = app.scroll_offset;

    // Check if the buffer position is visible
    if buffer_row < scroll_row || buffer_col < scroll_col {
        return None;
    }

    let relative_row = buffer_row - scroll_row;
    let relative_col = buffer_col - scroll_col;

    // Check if position is within visible area
    if relative_row >= editor_area.height as usize {
        return None;
    }

    // Account for line numbers with better width calculation
    let line_number_width = if app.get_line_numbers_setting() {
        if let Some(buffer) = app.buffers.get(app.active_buffer) {
            buffer.line_number_width()
        } else {
            0
        }
    } else {
        0
    };

    let screen_col = relative_col + line_number_width;

    // Check if cursor would be outside visible width
    if screen_col >= editor_area.width as usize {
        return None;
    }

    Some((
        editor_area.x + screen_col as u16,
        editor_area.y + relative_row as u16,
    ))
}
