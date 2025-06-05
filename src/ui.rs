use crate::widgets::cursor::CursorSupport;
use crate::widgets::editor::Editor;
use crate::widgets::modal::CommandPalette;
use crate::App;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph},
};

impl App {
    /// Main render function for the application UI
    pub fn render(&mut self, f: &mut Frame) {
        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(1),    // Editor area
                Constraint::Length(1), // Status line
            ])
            .split(f.area());

        // Render the editor
        self.render_editor(f, chunks[0]);

        // Render status line
        self.render_status_line(f, chunks[1]);

        // Render command line (only in normal mode, modal handles command input)
        if !self.show_command_palette {
            // self.render_command_line(f, chunks[2]);
        }

        // Update and render toast notifications
        self.toast_manager.update();
        if self.toast_manager.has_active_toasts() {
            self.render_toasts(f, f.area());
        }

        // Render command palette modal if active
        if self.show_command_palette {
            self.render_command_palette(f, f.area());
        }

        // Render the active cursor last
        self.render_active_cursor(f);
    }

    /// Render the main editor area
    fn render_editor(&mut self, f: &mut Frame, area: Rect) {
        if self.buffers.is_empty() {
            return;
        }

        // Get configuration for line numbers
        let show_line_numbers = self.get_line_numbers_setting();

        let editor = Editor {
            buffer: &self.buffers[self.active_buffer],
            scroll_offset: self.scroll_offset,
            show_line_numbers,
        };

        f.render_widget(editor, area);

        // Update cursor manager for editor context (but don't force cursor visibility)
        self.update_editor_cursor(area, show_line_numbers);
    }

    /// Render the status line
    fn render_status_line(&self, f: &mut Frame, area: Rect) {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let (row, col) = buffer.cursor_pos;

            // Check if there's a selection and include selection info
            let mut status = String::with_capacity(128); // Pre-allocate capacity
            status.push_str(&buffer.name);
            status.push_str(" | Ln ");
            status.push_str(&(row + 1).to_string());
            status.push_str(", Col ");
            status.push_str(&(col + 1).to_string());
            status.push_str(" | ");
            status.push_str(if buffer.modified { "Unsaved" } else { "Saved" });

            if let Some(selected_text) = buffer.get_selected_text() {
                let char_count = selected_text.len();
                let line_count = selected_text.matches('\n').count() + 1;
                status.push_str(" | Selection: ");
                if line_count > 1 {
                    status.push_str(&line_count.to_string());
                    status.push_str(" lines, ");
                    status.push_str(&char_count.to_string());
                    status.push_str(" chars");
                } else {
                    status.push_str(&char_count.to_string());
                    status.push_str(" chars");
                }
            }

            let block = Block::default()
                .style(Style::default().bg(Color::Black))
                .borders(Borders::NONE);

            let status_widget = Paragraph::new(Line::from(vec![Span::styled(
                status,
                Style::default().fg(Color::White).bg(Color::LightBlue),
            )]))
            .block(block);

            f.render_widget(status_widget, area);
        }
    }

    /// Render toast notifications
    fn render_toasts(&self, f: &mut Frame, area: Rect) {
        use crate::widgets::toast::ToastWidget;
        let toast_widget = ToastWidget::new(&self.toast_manager);
        f.render_widget(toast_widget, area);
    }

    /// Render command palette modal
    fn render_command_palette(&mut self, f: &mut Frame, area: Rect) {
        let palette = CommandPalette::new(&self.command_input);

        // Use the CursorSupport trait to calculate proper cursor position before rendering
        let cursor_position = palette.calculate_cursor_position(
            (self.command_input.len(), 0), // Cursor is at end of input
            area,
        );

        // Render the palette
        f.render_widget(palette, area);

        // Ensure only command palette cursor is active
        self.cursor_manager.hide_cursor("editor");
        self.cursor_manager.hide_cursor("file_search");
        self.cursor_manager.hide_cursor("text_search");
        self.cursor_manager.hide_cursor("command");

        self.cursor_manager.update_cursor_position(
            "command_palette",
            cursor_position.x,
            cursor_position.y,
        );
        self.cursor_manager.set_active_context("command_palette");
    }

    /// Get line numbers setting from config
    pub fn get_line_numbers_setting(&self) -> bool {
        let config_dir = &self.user_dir;
        if config_dir.exists() {
            let mut config_manager = crate::config::ConfigManager::new(config_dir);
            if config_manager.load().is_ok() {
                config_manager.get_config().editor.show_line_numbers
            } else {
                true // Default to showing line numbers if config can't be loaded
            }
        } else {
            true // Default to showing line numbers if config directory doesn't exist
        }
    }

    /// Ensure cursor is visible within the editor area (only call when cursor moves programmatically)
    pub fn ensure_cursor_visible(&mut self, area: Rect) {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let (row, col) = buffer.cursor_pos;
            let (scroll_row, scroll_col) = self.scroll_offset;

            // Adjust vertical scroll if needed (no borders)
            let visible_rows = area.height as usize;

            // Only change scroll position if the cursor is outside the visible area
            // This prevents the "snapping back" effect when scrolling manually
            if row < scroll_row {
                // Cursor is above visible area - scroll up just enough to show it
                self.scroll_offset.0 = row;
            } else if row >= scroll_row + visible_rows {
                // Cursor is below visible area - scroll down just enough to show it
                // Don't subtract 1 to avoid the snapping behavior
                self.scroll_offset.0 = row.saturating_sub(visible_rows) + 1;
            }
            // Otherwise, don't change vertical scroll (allows manual scrolling)

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

    /// Update cursor position for the editor context
    fn update_editor_cursor(&mut self, area: Rect, show_line_numbers: bool) {
        // Don't update editor cursor if command palette is open
        if self.show_command_palette {
            self.cursor_manager.hide_cursor("editor");
            return;
        }

        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let (row, col) = buffer.cursor_pos;
            let (scroll_row, scroll_col) = self.scroll_offset;

            // Calculate line number width for cursor positioning
            let line_number_width = if show_line_numbers {
                buffer.line_number_width() as u16
            } else {
                0
            };

            let cursor_x = (col.saturating_sub(scroll_col)) as u16 + line_number_width;
            let cursor_y = (row.saturating_sub(scroll_row)) as u16;

            // Always update cursor position, but clip it to the visible area
            // This ensures the scroll logic can work properly
            let absolute_x = area.x + cursor_x.min(area.width.saturating_sub(1));
            let absolute_y = area.y + cursor_y.min(area.height.saturating_sub(1));

            // Only show cursor if it's actually within the visible area
            let is_visible = cursor_y < area.height && cursor_x < area.width;

            // Ensure only editor cursor is active
            self.cursor_manager.hide_cursor("command_palette");
            self.cursor_manager.hide_cursor("file_search");
            self.cursor_manager.hide_cursor("text_search");
            self.cursor_manager.hide_cursor("command");

            // Always update the cursor position in the manager, even if not visible
            // This ensures position is maintained when scrolling
            self.cursor_manager
                .update_cursor_position("editor", absolute_x, absolute_y);

            if is_visible {
                self.cursor_manager.set_active_context("editor");
            } else {
                // Hide the cursor if outside visible area, but maintain its position
                self.cursor_manager.hide_cursor("editor");
            }
        }
    }

    /// Render active cursor from cursor manager
    fn render_active_cursor(&mut self, f: &mut Frame) {
        // Only render the active cursor context - ensure all others are hidden
        if let Some(active_context) = self
            .cursor_manager
            .get_active_context()
            .map(|s| s.to_string())
        {
            // Explicitly hide all non-active cursors first
            let all_contexts = [
                "editor",
                "command_palette",
                "file_search",
                "text_search",
                "command",
            ];
            for context in &all_contexts {
                if *context != active_context {
                    self.cursor_manager.hide_cursor(context);
                }
            }

            // Only render if the cursor is visible and we have a position
            if let Some(position) = self.cursor_manager.get_cursor_position(&active_context) {
                use crate::widgets::Cursor;

                let cursor = Cursor::new(active_context.clone())
                    .with_position(position.x, position.y)
                    .with_style(Style::default().bg(Color::White).fg(Color::Black))
                    .active(true);

                // Get the cursor state from the manager
                if let Some(cursor_state) =
                    self.cursor_manager.get_cursor_state_mut(&active_context)
                {
                    // Render the cursor widget on the entire screen area
                    f.render_stateful_widget(cursor, f.area(), cursor_state);
                }
            }
        }
    }
}
