use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, Widget},
};

use crate::buffer::Buffer;

pub struct Editor<'a> {
    pub buffer: &'a Buffer,
    pub scroll_offset: (usize, usize), // (row, col) offset for viewport scrolling
    pub show_line_numbers: bool,       // Whether to display line numbers
}

impl<'a> Editor<'a> {
    pub fn new(buffer: &'a Buffer) -> Self {
        Self {
            buffer,
            scroll_offset: (0, 0),
            show_line_numbers: true, // Enable line numbers by default
        }
    }

    pub fn ensure_cursor_visible(&mut self, area: Rect) {
        let (row, col) = self.buffer.cursor_pos;
        let (scroll_row, scroll_col) = self.scroll_offset;

        // Adjust vertical scroll if needed (no borders now, so use full height)
        let visible_rows = area.height as usize;
        if row < scroll_row {
            self.scroll_offset.0 = row;
        } else if row >= scroll_row + visible_rows {
            self.scroll_offset.0 = row.saturating_sub(visible_rows) + 1;
        }

        // Adjust horizontal scroll if needed (account for line numbers)
        let line_number_width = if self.show_line_numbers {
            // Use consistent width based on total buffer size - count digits efficiently
            let buffer_lines = self.buffer.content.len().max(1);
            let mut digits = 1;
            let mut n = buffer_lines;
            while n >= 10 {
                digits += 1;
                n /= 10;
            }
            digits + 1 // +1 for spacing
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

impl Widget for Editor<'_> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        // No borders - use the full area for content
        let inner_area = area;

        // Determine visible portion of the buffer
        let start_row = self.scroll_offset.0;
        let end_row = (start_row + inner_area.height as usize).min(self.buffer.content.len());
        let h_offset = self.scroll_offset.1;

        // Calculate line number width (if enabled)
        let line_number_width = if self.show_line_numbers {
            // Use consistent width based on total buffer size, not visible area
            // This prevents shifting when scrolling - count digits efficiently
            let total_lines = self.buffer.content.len().max(1);
            let mut digits = 1;
            let mut n = total_lines;
            while n >= 10 {
                digits += 1;
                n /= 10;
            }
            digits + 1 // +1 for spacing
        } else {
            0
        };

        // Render visible lines
        let mut lines = Vec::new();

        // Get selection range for rendering highlighting
        let selection_range = self.buffer.get_selection_range();

        for i in start_row..end_row {
            if let Some(line) = self.buffer.content.get(i) {
                // Extract the visible portion of the line without cloning
                let visible_content = if h_offset < line.len() {
                    &line[h_offset..]
                } else {
                    ""
                };

                // Create spans for the line content, with highlighting for selection
                let content_spans = if let Some((start, end)) = selection_range {
                    let mut spans = Vec::new();

                    // Check if this line is within selection
                    if i < start.row || i > end.row {
                        // Line is completely outside selection
                        spans.push(Span::raw(visible_content));
                    } else if i == start.row && i == end.row {
                        // Selection starts and ends on this line
                        let start_col = start.col.saturating_sub(h_offset);
                        let end_col = end.col.saturating_sub(h_offset);

                        // Text before selection
                        if start_col > 0 && start_col <= visible_content.len() {
                            spans.push(Span::raw(&visible_content[..start_col]));
                        }

                        // Selected text
                        if start_col < visible_content.len() && end_col > 0 {
                            let sel_start = start_col;
                            let sel_end = end_col.min(visible_content.len());
                            if sel_end > sel_start {
                                spans.push(Span::styled(
                                    &visible_content[sel_start..sel_end],
                                    Style::default().bg(Color::DarkGray).fg(Color::White),
                                ));
                            }
                        }

                        // Text after selection
                        if end_col < visible_content.len() {
                            spans.push(Span::raw(&visible_content[end_col..]));
                        }
                    } else if i == start.row {
                        // First line of multi-line selection
                        let start_col = start.col.saturating_sub(h_offset);

                        // Text before selection
                        if start_col > 0 && start_col <= visible_content.len() {
                            spans.push(Span::raw(&visible_content[..start_col]));
                        }

                        // Selected text to end of line
                        if start_col < visible_content.len() {
                            spans.push(Span::styled(
                                &visible_content[start_col..],
                                Style::default().bg(Color::DarkGray).fg(Color::White),
                            ));
                        }
                    } else if i == end.row {
                        // Last line of multi-line selection
                        let end_col = end.col.saturating_sub(h_offset);

                        // Selected text from start of line to end of selection
                        if end_col > 0 {
                            let sel_end = end_col.min(visible_content.len());
                            spans.push(Span::styled(
                                &visible_content[..sel_end],
                                Style::default().bg(Color::DarkGray).fg(Color::White),
                            ));
                        }

                        // Text after selection
                        if end_col < visible_content.len() {
                            spans.push(Span::raw(&visible_content[end_col..]));
                        }
                    } else {
                        // Middle line of multi-line selection - whole line is selected
                        spans.push(Span::styled(
                            visible_content,
                            Style::default().bg(Color::DarkGray).fg(Color::White),
                        ));
                    }

                    spans
                } else {
                    // No selection, just show the regular text
                    vec![Span::raw(visible_content)]
                };

                if self.show_line_numbers {
                    // Create line with line number
                    let line_num = i + 1; // 1-indexed line numbers
                    let line_num_str =
                        format!("{:>width$}", line_num, width = line_number_width - 1);

                    // Combine line number with content spans
                    let mut line_spans = vec![
                        Span::styled(line_num_str, Style::default().fg(Color::Rgb(100, 100, 120))),
                        Span::raw(" "), // Separator
                    ];
                    line_spans.extend(content_spans);

                    lines.push(Line::from(line_spans));
                } else {
                    lines.push(Line::from(content_spans));
                }
            } else {
                // For empty lines, still need to maintain line numbers if enabled
                if self.show_line_numbers {
                    let line_num = i + 1; // 1-indexed line numbers
                    let line_num_str =
                        format!("{:>width$}", line_num, width = line_number_width - 1);

                    lines.push(Line::from(vec![
                        Span::styled(line_num_str, Style::default().fg(Color::Rgb(100, 100, 120))),
                        Span::raw(" "), // Separator
                        Span::raw(""),
                    ]));
                } else {
                    lines.push(Line::from(""));
                }
            }
        }

        // Create paragraph with all visible lines (no block, just content)
        let paragraph =
            Paragraph::new(lines).style(Style::default().fg(Color::White).bg(Color::Black));
        paragraph.render(inner_area, buf);

        // Position cursor
        let cursor_row = self.buffer.cursor_pos.0.saturating_sub(start_row) as u16;
        let cursor_col = self.buffer.cursor_pos.1.saturating_sub(h_offset) as u16;

        // For cursor positioning, we need to consider line number width when show_line_numbers is true
        let effective_cursor_col = if self.show_line_numbers {
            cursor_col + line_number_width as u16
        } else {
            cursor_col
        };

        if cursor_row < inner_area.height && effective_cursor_col < inner_area.width {
            // Note: In newer Ratatui versions, the cursor is set at the app level
        }
    }
}

// Implementation for a stateful widget version if needed later
impl StatefulWidget for Editor<'_> {
    type State = ();

    fn render(self, area: Rect, buf: &mut TuiBuffer, _state: &mut Self::State) {
        Widget::render(self, area, buf);
    }
}
