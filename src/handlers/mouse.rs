/// Mouse input handlers that subscribe to mouse events
use crate::events::{AppEvent, EventBus};
use crate::{App, CommandMode};
use anyhow::Result;
use ratatui::crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

/// Mouse handler that processes mouse events
pub struct MouseHandler {
    app_state: Arc<RwLock<App>>,
    event_sender: mpsc::UnboundedSender<AppEvent>,
}

impl MouseHandler {
    /// Create a new mouse handler
    pub fn new(app_state: Arc<RwLock<App>>, event_sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            app_state,
            event_sender,
        }
    }

    /// Subscribe to mouse events
    pub async fn subscribe(&self, event_bus: &EventBus) -> Result<()> {
        let handler = MouseHandler::new(self.app_state.clone(), self.event_sender.clone());

        event_bus
            .subscribe_async("mouse_input", move |event| {
                let handler = handler.clone();
                async move { handler.handle_mouse_event(event).await }
            })
            .await;

        Ok(())
    }

    /// Handle mouse events
    async fn handle_mouse_event(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::MouseInput(mouse) = event {
            let app = self.app_state.read().await;
            let command_mode = app.command_mode.clone();
            drop(app);

            match command_mode {
                CommandMode::Normal => self.handle_normal_mode_mouse(mouse).await?,
                CommandMode::Command => self.handle_command_mode_mouse(mouse).await?,
                CommandMode::FileSearch => self.handle_file_search_mode_mouse(mouse).await?,
                CommandMode::TextSearch => self.handle_text_search_mode_mouse(mouse).await?,
            }
        }

        Ok(())
    }

    /// Handle mouse events in normal editing mode
    async fn handle_normal_mode_mouse(&self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                self.handle_click(mouse.column, mouse.row).await?;
            }
            MouseEventKind::Drag(MouseButton::Left) => {
                self.handle_drag(mouse.column, mouse.row).await?;
            }
            MouseEventKind::Up(MouseButton::Left) => {
                self.handle_release(mouse.column, mouse.row).await?;
            }
            MouseEventKind::ScrollUp => {
                self.handle_scroll(-8).await?; // Scroll 8 lines up
            }
            MouseEventKind::ScrollDown => {
                self.handle_scroll(8).await?; // Scroll 8 lines down
            }
            MouseEventKind::Down(MouseButton::Right) => {
                static RIGHT_CLICK_MSG: &str = "Right click detected";
                self.event_sender.send(AppEvent::StatusMessage {
                    message: RIGHT_CLICK_MSG.into(),
                })?;
            }
            MouseEventKind::Down(MouseButton::Middle) => {
                static MIDDLE_CLICK_MSG: &str = "Middle click detected";
                self.event_sender.send(AppEvent::StatusMessage {
                    message: MIDDLE_CLICK_MSG.into(),
                })?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle mouse click to position cursor
    async fn handle_click(&self, mouse_x: u16, mouse_y: u16) -> Result<()> {
        let mut app = self.app_state.write().await;

        // Get actual terminal size
        let (terminal_width, terminal_height) =
            if let Ok((w, h)) = ratatui::crossterm::terminal::size() {
                (w, h)
            } else {
                (120, 30) // Fallback
            };

        let editor_area = ratatui::layout::Rect {
            x: 0,
            y: 0,
            width: terminal_width,
            height: terminal_height.saturating_sub(1), // -1 for status line
        };

        // Check if click is within editor area
        if mouse_y >= editor_area.height {
            // Click is in status line or below - ignore
            return Ok(());
        }

        // Convert screen coordinates to buffer coordinates using proper conversion
        if let Some((buffer_row, buffer_col)) =
            crate::input::coordinates::screen_to_buffer_coords(&app, mouse_x, mouse_y)
        {
            let active_buffer = app.active_buffer;

            if let Some(buffer) = app.buffers.get_mut(active_buffer) {
                // Clear any existing selection
                buffer.clear_selection();

                // Position cursor at click location
                buffer.cursor_pos = (buffer_row, buffer_col);

                // Start potential drag selection
                app.mouse_drag_start = Some((buffer_row, buffer_col));

                // Ensure clicked position is visible
                // (scroll will be adjusted in the render cycle)
            }

            // Publish cursor moved event
            self.event_sender.send(AppEvent::BufferCursorMoved {
                buffer_id: 0,
                row: buffer_row,
                col: buffer_col,
            })?;

            // Pre-allocate string for cursor position message
            let mut cursor_msg = String::with_capacity(32);
            cursor_msg.push_str("Cursor positioned at ");
            cursor_msg.push_str(&(buffer_row + 1).to_string());
            cursor_msg.push(':');
            cursor_msg.push_str(&(buffer_col + 1).to_string());

            self.event_sender.send(AppEvent::StatusMessage {
                message: cursor_msg.into(),
            })?;
        }

        Ok(())
    }

    /// Handle mouse drag for text selection
    async fn handle_drag(&self, mouse_x: u16, mouse_y: u16) -> Result<()> {
        let mut app = self.app_state.write().await;

        // Convert screen coordinates to buffer coordinates using proper conversion
        if let Some((buffer_row, buffer_col)) =
            crate::input::coordinates::screen_to_buffer_coords(&app, mouse_x, mouse_y)
        {
            let active_buffer = app.active_buffer;
            let mouse_drag_start = app.mouse_drag_start;

            if let Some(buffer) = app.buffers.get_mut(active_buffer) {
                if let Some(start_pos) = mouse_drag_start {
                    // Enable visual mode if not already enabled
                    if !buffer.visual_mode {
                        buffer.visual_mode = true;
                        buffer.selection_start = Some(start_pos);
                    }

                    // Update cursor position to drag end
                    buffer.cursor_pos = (buffer_row, buffer_col);
                }
            }

            // Publish selection changed event
            self.event_sender.send(AppEvent::BufferSelectionChanged {
                buffer_id: 0,
                start: mouse_drag_start,
                end: Some((buffer_row, buffer_col)),
            })?;
        }

        Ok(())
    }

    /// Handle mouse button release
    async fn handle_release(&self, _mouse_x: u16, _mouse_y: u16) -> Result<()> {
        let mut app = self.app_state.write().await;

        // Clear drag start - selection is finalized
        app.mouse_drag_start = None;

        // Show selection info if we have one
        if let Some(buffer) = app.buffers.get(app.active_buffer) {
            if let Some(selected_text) = buffer.get_selected_text() {
                let char_count = selected_text.chars().count();
                let line_count = selected_text.lines().count();
                drop(app);

                // Pre-allocate string for selection message
                let mut selection_msg = String::with_capacity(64);
                selection_msg.push_str("Selected ");
                selection_msg.push_str(&char_count.to_string());
                selection_msg.push_str(" characters across ");
                selection_msg.push_str(&line_count.to_string());
                selection_msg.push_str(" lines");

                self.event_sender.send(AppEvent::StatusMessage {
                    message: selection_msg.into(),
                })?;
            }
        }

        Ok(())
    }

    /// Handle scroll events
    async fn handle_scroll(&self, delta: i32) -> Result<()> {
        let mut app = self.app_state.write().await;

        let (current_row, current_col) = app.scroll_offset;

        // Get terminal dimensions
        let term_height = if let Ok((_, h)) = ratatui::crossterm::terminal::size() {
            h
        } else {
            30 // Fallback size
        };

        // Calculate visible rows in editor (terminal height minus status bar)
        let visible_rows = term_height.saturating_sub(1) as usize;

        // Get buffer size and calculate maximum scroll position
        let max_scroll_row = if let Some(buffer) = app.buffers.get(app.active_buffer) {
            // Allow scrolling to show the last line at the bottom of the editor
            // This means max scroll is buffer size minus visible rows
            buffer.content.len().saturating_sub(visible_rows / 2) // Allows more scrolling past the end
        } else {
            0
        };

        if delta > 0 {
            // Scroll down - don't scroll past the calculated maximum
            let new_row = (current_row + delta as usize).min(max_scroll_row);
            app.scroll_offset = (new_row, current_col);
        } else {
            // Scroll up - don't scroll above the beginning (line 0)
            let new_row = current_row.saturating_sub((-delta) as usize);
            app.scroll_offset = (new_row, current_col);
        }

        // Send status message showing current scroll position
        let mut scroll_msg = String::with_capacity(32);
        scroll_msg.push_str("Scrolled to line ");
        scroll_msg.push_str(&(app.scroll_offset.0 + 1).to_string());

        self.event_sender.send(AppEvent::StatusMessage {
            message: scroll_msg.into(),
        })?;

        Ok(())
    }

    /// Handle mouse events in command palette mode
    async fn handle_command_mode_mouse(&self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // TODO: Check if click is within command palette area
                // For now, just close the palette if clicked anywhere
                if !self
                    .is_click_in_command_palette(mouse.column, mouse.row)
                    .await
                {
                    self.close_command_palette().await?;
                }
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle mouse events in file search mode
    async fn handle_file_search_mode_mouse(&self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Click outside search - return to normal mode
                static NORMAL_MODE: &str = "normal";
                static FILE_SEARCH_CONTEXT: &str = "file_search";
                static EDITOR_CONTEXT: &str = "editor";

                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: NORMAL_MODE.into(),
                })?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: FILE_SEARCH_CONTEXT.into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: EDITOR_CONTEXT.into(),
                })?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle mouse events in text search mode
    async fn handle_text_search_mode_mouse(&self, mouse: MouseEvent) -> Result<()> {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                // Click outside search - return to normal mode
                static NORMAL_MODE: &str = "normal";
                static TEXT_SEARCH_CONTEXT: &str = "text_search";
                static EDITOR_CONTEXT: &str = "editor";

                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: NORMAL_MODE.into(),
                })?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: TEXT_SEARCH_CONTEXT.into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: EDITOR_CONTEXT.into(),
                })?;
            }
            _ => {}
        }

        Ok(())
    }

    /// Check if a click is within the command palette area
    async fn is_click_in_command_palette(&self, mouse_x: u16, mouse_y: u16) -> bool {
        // Get actual terminal size
        let (screen_width, screen_height) = if let Ok((w, h)) = ratatui::crossterm::terminal::size()
        {
            (w, h)
        } else {
            (120, 30) // Fallback
        };

        let modal_width = screen_width * 60 / 100;
        let modal_height = 3;

        let modal_x = (screen_width - modal_width) / 2;
        let modal_y = (screen_height - modal_height) / 2;

        mouse_x >= modal_x
            && mouse_x < modal_x + modal_width
            && mouse_y >= modal_y
            && mouse_y < modal_y + modal_height
    }

    /// Close command palette and return to normal mode
    async fn close_command_palette(&self) -> Result<()> {
        static NORMAL_MODE: &str = "normal";
        static COMMAND_PALETTE_CONTEXT: &str = "command_palette";
        static EDITOR_CONTEXT: &str = "editor";

        self.event_sender.send(AppEvent::HideCommandPalette)?;
        self.event_sender.send(AppEvent::ModeChanged {
            new_mode: NORMAL_MODE.into(),
        })?;
        self.event_sender.send(AppEvent::CursorHide {
            context: COMMAND_PALETTE_CONTEXT.into(),
        })?;
        self.event_sender.send(AppEvent::CursorShow {
            context: EDITOR_CONTEXT.into(),
        })?;
        Ok(())
    }
}

impl Clone for MouseHandler {
    fn clone(&self) -> Self {
        Self {
            app_state: self.app_state.clone(),
            event_sender: self.event_sender.clone(),
        }
    }
}
