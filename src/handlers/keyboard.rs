use crate::events::{AppEvent, EventBus};
use crate::{App, CommandMode};
use anyhow::Result;
use ratatui::crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::sync::mpsc;

/// Keyboard handler that processes keyboard events
pub struct KeyboardHandler {
    app_state: Arc<RwLock<App>>,
    event_sender: mpsc::UnboundedSender<AppEvent>,
}

impl KeyboardHandler {
    /// Create a new keyboard handler
    pub fn new(app_state: Arc<RwLock<App>>, event_sender: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            app_state,
            event_sender,
        }
    }

    /// Subscribe to keyboard events
    pub async fn subscribe(&self, event_bus: &EventBus) -> Result<()> {
        let handler = KeyboardHandler::new(self.app_state.clone(), self.event_sender.clone());

        event_bus
            .subscribe_async("key_input", move |event| {
                let handler = handler.clone();
                async move { handler.handle_key_event(event).await }
            })
            .await;

        Ok(())
    }

    /// Handle keyboard events
    async fn handle_key_event(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::KeyInput(key) = event {
            let app = self.app_state.read().await;
            let command_mode = app.command_mode.clone();
            drop(app); // Release read lock early

            match command_mode {
                CommandMode::Normal => self.handle_normal_mode_key(key).await?,
                CommandMode::Command => self.handle_command_mode_key(key).await?,
                CommandMode::FileSearch => self.handle_file_search_key(key).await?,
                CommandMode::TextSearch => self.handle_text_search_key(key).await?,
            }
        }

        Ok(())
    }

    /// Handle keyboard input in normal mode
    async fn handle_normal_mode_key(&self, key: KeyEvent) -> Result<()> {
        // Check for key combinations first
        match (key.code, key.modifiers) {
            (KeyCode::Char('p'), KeyModifiers::ALT) => {
                // Open command palette with Alt+P
                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: "command".into(),
                })?;
                self.event_sender.send(AppEvent::ShowCommandPalette)?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: "editor".into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: "command_palette".into(),
                })?;
            }
            (KeyCode::Char('q'), KeyModifiers::CONTROL) => {
                // Quit with Ctrl+Q
                self.event_sender.send(AppEvent::Quit)?;
            }
            (KeyCode::Char('s'), KeyModifiers::CONTROL) => {
                // Save with Ctrl+S
                self.handle_save_command().await?;
            }
            (KeyCode::Char('o'), KeyModifiers::CONTROL) => {
                // Open file with Ctrl+O
                self.handle_open_command().await?;
            }
            (KeyCode::Char('n'), KeyModifiers::CONTROL) => {
                // New buffer with Ctrl+N
                self.handle_new_buffer().await?;
            }
            (KeyCode::Char('v'), KeyModifiers::CONTROL) => {
                // Toggle visual mode with Ctrl+V
                self.handle_toggle_visual_mode().await?;
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                // Copy with Ctrl+C
                self.handle_copy().await?;
            }
            (KeyCode::Char('x'), KeyModifiers::CONTROL) => {
                // Cut with Ctrl+X
                self.handle_cut().await?;
            }
            (KeyCode::Tab, KeyModifiers::NONE) => {
                // Next buffer with Tab
                self.handle_next_buffer().await?;
            }
            (KeyCode::Tab, KeyModifiers::SHIFT) => {
                // Previous buffer with Shift+Tab
                self.handle_prev_buffer().await?;
            }
            (KeyCode::Esc, _) => {
                self.handle_escape().await?;
            }
            // Movement keys
            (KeyCode::Up, modifiers) => {
                self.handle_cursor_movement(crate::buffer::CursorMovement::Up, modifiers)
                    .await?;
            }
            (KeyCode::Down, modifiers) => {
                self.handle_cursor_movement(crate::buffer::CursorMovement::Down, modifiers)
                    .await?;
            }
            (KeyCode::Left, modifiers) => {
                self.handle_cursor_movement(crate::buffer::CursorMovement::Left, modifiers)
                    .await?;
            }
            (KeyCode::Right, modifiers) => {
                self.handle_cursor_movement(crate::buffer::CursorMovement::Right, modifiers)
                    .await?;
            }
            (KeyCode::Home, modifiers) => {
                let movement = if modifiers.contains(KeyModifiers::CONTROL) {
                    crate::buffer::CursorMovement::BufferStart
                } else {
                    crate::buffer::CursorMovement::LineStart
                };
                self.handle_cursor_movement(movement, modifiers).await?;
            }
            (KeyCode::End, modifiers) => {
                let movement = if modifiers.contains(KeyModifiers::CONTROL) {
                    crate::buffer::CursorMovement::BufferEnd
                } else {
                    crate::buffer::CursorMovement::LineEnd
                };
                self.handle_cursor_movement(movement, modifiers).await?;
            }
            (KeyCode::PageUp, modifiers) => {
                self.handle_cursor_movement(crate::buffer::CursorMovement::PageUp, modifiers)
                    .await?;
            }
            (KeyCode::PageDown, modifiers) => {
                self.handle_cursor_movement(crate::buffer::CursorMovement::PageDown, modifiers)
                    .await?;
            }
            // Text input
            (KeyCode::Char(c), KeyModifiers::NONE) => {
                self.handle_char_input(c).await?;
            }
            (KeyCode::Enter, KeyModifiers::NONE) => {
                self.handle_enter().await?;
            }
            (KeyCode::Backspace, KeyModifiers::NONE) => {
                self.handle_backspace().await?;
            }
            (KeyCode::Delete, KeyModifiers::NONE) => {
                self.handle_delete().await?;
            }
            _ => {} // Ignore other key combinations
        }

        Ok(())
    }

    /// Handle escape key
    async fn handle_escape(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if buffer.visual_mode {
                buffer.clear_selection();
                drop(app);
                self.event_sender.send(AppEvent::StatusMessage {
                    message: "Selection cleared".into(),
                })?;
            }
        }

        Ok(())
    }

    /// Handle cursor movement
    async fn handle_cursor_movement(
        &self,
        movement: crate::buffer::CursorMovement,
        modifiers: KeyModifiers,
    ) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if modifiers.contains(KeyModifiers::SHIFT) && !buffer.visual_mode {
                buffer.toggle_visual_mode();
            }
            buffer.move_cursor(movement);

            let (row, col) = buffer.cursor_pos;
            drop(app);
            self.event_sender.send(AppEvent::BufferCursorMoved {
                buffer_id: 0,
                row,
                col,
            })?;
        }

        Ok(())
    }

    /// Handle character input
    async fn handle_char_input(&self, c: char) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if buffer.visual_mode {
                buffer.delete_selection();
                buffer.visual_mode = false;
                buffer.selection_start = None;
            }

            buffer.insert_char(c);

            let content: Arc<str> = buffer.content_as_string().into();
            drop(app);
            self.event_sender.send(AppEvent::BufferChanged {
                buffer_id: 0,
                content,
            })?;
        }

        Ok(())
    }

    /// Handle enter key
    async fn handle_enter(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if buffer.visual_mode {
                buffer.delete_selection();
                buffer.visual_mode = false;
                buffer.selection_start = None;
            }
            buffer.insert_newline();

            let content: Arc<str> = buffer.content_as_string().into();
            drop(app);
            self.event_sender.send(AppEvent::BufferChanged {
                buffer_id: 0,
                content,
            })?;
        }

        Ok(())
    }

    /// Handle backspace key
    async fn handle_backspace(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if buffer.visual_mode {
                buffer.delete_selection();
                buffer.visual_mode = false;
                buffer.selection_start = None;
            } else {
                buffer.backspace();
            }

            let content: Arc<str> = buffer.content_as_string().into();
            drop(app);
            self.event_sender.send(AppEvent::BufferChanged {
                buffer_id: 0,
                content,
            })?;
        }

        Ok(())
    }

    /// Handle delete key
    async fn handle_delete(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if buffer.visual_mode {
                buffer.delete_selection();
                buffer.visual_mode = false;
                buffer.selection_start = None;
            } else {
                buffer.delete();
            }

            let content: Arc<str> = buffer.content_as_string().into();
            drop(app);
            self.event_sender.send(AppEvent::BufferChanged {
                buffer_id: 0,
                content,
            })?;
        }

        Ok(())
    }

    /// Handle keyboard input in command mode
    async fn handle_command_mode_key(&self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                // Close command palette
                self.event_sender.send(AppEvent::HideCommandPalette)?;
                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: "normal".into(),
                })?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: "command_palette".into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: "editor".into(),
                })?;
            }
            KeyCode::Enter => {
                // Execute command
                let command = {
                    let app = self.app_state.read().await;
                    app.command_input.clone()
                };

                if !command.is_empty() {
                    // Execute the command
                    self.execute_command(&command).await?;
                }

                // Close command palette
                self.event_sender.send(AppEvent::HideCommandPalette)?;
                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: "normal".into(),
                })?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: "command_palette".into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: "editor".into(),
                })?;
            }
            KeyCode::Char(c) => {
                // Add character to command input
                let mut app = self.app_state.write().await;
                app.command_input.push(c);
            }
            KeyCode::Backspace => {
                // Remove character from command input
                let mut app = self.app_state.write().await;
                app.command_input.pop();
            }
            _ => {}
        }

        Ok(())
    }

    /// Handle keyboard input in file search mode
    async fn handle_file_search_key(&self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: "normal".into(),
                })?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: "file_search".into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: "editor".into(),
                })?;
            }
            _ => {
                todo!("Implement file search handling");
            }
        }

        Ok(())
    }

    /// Handle keyboard input in text search mode
    async fn handle_text_search_key(&self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Esc => {
                self.event_sender.send(AppEvent::ModeChanged {
                    new_mode: "normal".into(),
                })?;
                self.event_sender.send(AppEvent::CursorHide {
                    context: "text_search".into(),
                })?;
                self.event_sender.send(AppEvent::CursorShow {
                    context: "editor".into(),
                })?;
            }
            _ => {
                // TODO: Implement text search handling
            }
        }

        Ok(())
    }

    /// Handle save command (Ctrl+S)
    async fn handle_save_command(&self) -> Result<()> {
        let app = self.app_state.read().await;
        let active_buffer = app.active_buffer;
        if let Some(buffer) = app.buffers.get(active_buffer) {
            if let Some(path) = &buffer.path {
                let content = buffer.content_as_string();
                let path = path.clone();
                drop(app);

                // Save asynchronously
                if let Err(e) = tokio::fs::write(&path, content).await {
                    let error_msg = format!("Error saving file: {}", e);
                    self.event_sender.send(AppEvent::StatusMessage {
                        message: error_msg.into(),
                    })?;
                } else {
                    // Mark buffer as clean
                    let mut app = self.app_state.write().await;
                    if let Some(buffer) = app.buffers.get_mut(active_buffer) {
                        buffer.modified = false;
                    }
                    drop(app);

                    let success_message = format!("File saved: {}", path.display());
                    let success_msg: Arc<str> = success_message.into();
                    self.event_sender.send(AppEvent::ToastMessage {
                        message: success_msg.clone(),
                        toast_type: "success".into(),
                    })?;
                    self.event_sender.send(AppEvent::StatusMessage {
                        message: success_msg,
                    })?;
                }
            } else {
                drop(app);
                self.event_sender.send(AppEvent::StatusMessage {
                    message: "No file path - use save as command".into(),
                })?;
            }
        }
        Ok(())
    }

    /// Handle open command (Ctrl+O) - opens command palette with open command
    async fn handle_open_command(&self) -> Result<()> {
        // Switch to command mode and pre-fill with "open "
        let mut app = self.app_state.write().await;
        app.command_mode = CommandMode::Command;
        app.command_input = "open ".to_string();
        app.show_command_palette = true;
        drop(app);

        self.event_sender.send(AppEvent::ModeChanged {
            new_mode: "command".into(),
        })?;
        self.event_sender.send(AppEvent::ShowCommandPalette)?;
        self.event_sender.send(AppEvent::CursorHide {
            context: "editor".into(),
        })?;
        self.event_sender.send(AppEvent::CursorShow {
            context: "command_palette".into(),
        })?;
        Ok(())
    }

    /// Handle new buffer command (Ctrl+N)
    async fn handle_new_buffer(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let new_buffer = crate::buffer::Buffer::new();
        app.add_buffer(new_buffer);
        drop(app);

        self.event_sender.send(AppEvent::StatusMessage {
            message: "New buffer created".into(),
        })?;
        Ok(())
    }

    /// Handle toggle visual mode (Ctrl+V)
    async fn handle_toggle_visual_mode(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            buffer.toggle_visual_mode();
            let message = if buffer.visual_mode {
                "Visual mode enabled"
            } else {
                "Visual mode disabled"
            };
            drop(app);

            self.event_sender.send(AppEvent::StatusMessage {
                message: message.into(),
            })?;
        }
        Ok(())
    }

    /// Handle copy command (Ctrl+C)
    async fn handle_copy(&self) -> Result<()> {
        let app = self.app_state.read().await;
        if let Some(buffer) = app.buffers.get(app.active_buffer) {
            if let Some(selected_text) = buffer.get_selected_text() {
                drop(app);
                // TODO: Implement clipboard integration
                let copy_msg = format!("Copied {} characters", selected_text.len());
                self.event_sender.send(AppEvent::StatusMessage {
                    message: copy_msg.into(),
                })?;
            } else {
                drop(app);
                self.event_sender.send(AppEvent::StatusMessage {
                    message: "No text selected".into(),
                })?;
            }
        }
        Ok(())
    }

    /// Handle cut command (Ctrl+X)
    async fn handle_cut(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let active_buffer = app.active_buffer;

        if let Some(buffer) = app.buffers.get_mut(active_buffer) {
            if let Some(selected_text) = buffer.get_selected_text() {
                // Delete the selection
                buffer.delete_selection();
                buffer.visual_mode = false;
                buffer.selection_start = None;

                let content: Arc<str> = buffer.content_as_string().into();
                drop(app);

                // TODO: Implement clipboard integration
                self.event_sender.send(AppEvent::BufferChanged {
                    buffer_id: 0,
                    content,
                })?;
                let cut_msg = format!("Cut {} characters", selected_text.len());
                self.event_sender.send(AppEvent::StatusMessage {
                    message: cut_msg.into(),
                })?;
            } else {
                drop(app);
                self.event_sender.send(AppEvent::StatusMessage {
                    message: "No text selected".into(),
                })?;
            }
        }
        Ok(())
    }

    /// Handle next buffer (Tab)
    async fn handle_next_buffer(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let buffer_count = app.buffers.len();
        if buffer_count > 1 {
            app.active_buffer = (app.active_buffer + 1) % buffer_count;
            app.scroll_offset = (0, 0);
            let buffer_name = app.buffers[app.active_buffer].name.clone();
            drop(app);

            let switch_msg = format!("Switched to buffer: {}", buffer_name);
            self.event_sender.send(AppEvent::StatusMessage {
                message: switch_msg.into(),
            })?;
        }
        Ok(())
    }

    /// Handle previous buffer (Shift+Tab)
    async fn handle_prev_buffer(&self) -> Result<()> {
        let mut app = self.app_state.write().await;
        let buffer_count = app.buffers.len();
        if buffer_count > 1 {
            app.active_buffer = if app.active_buffer == 0 {
                buffer_count - 1
            } else {
                app.active_buffer - 1
            };
            app.scroll_offset = (0, 0);
            let buffer_name = app.buffers[app.active_buffer].name.clone();
            drop(app);

            let switch_msg = format!("Switched to buffer: {}", buffer_name);
            self.event_sender.send(AppEvent::StatusMessage {
                message: switch_msg.into(),
            })?;
        }
        Ok(())
    }

    /// Execute a command from the command palette
    async fn execute_command(&self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.trim().split_whitespace().collect();
        if parts.is_empty() {
            return Ok(());
        }

        match parts[0] {
            "quit" | "q" => {
                self.event_sender.send(AppEvent::Quit)?;
            }
            "save" | "w" => {
                self.handle_save_command().await?;
            }
            "new" => {
                self.handle_new_buffer().await?;
            }
            "open" | "o" => {
                if parts.len() > 1 {
                    let file_path = parts[1..].join(" ");
                    self.handle_open_file(&file_path).await?;
                } else {
                    self.event_sender.send(AppEvent::StatusMessage {
                        message: "Usage: open <file_path>".into(),
                    })?;
                }
            }
            "next" | "n" => {
                self.handle_next_buffer().await?;
            }
            "prev" | "p" => {
                self.handle_prev_buffer().await?;
            }
            "toggle_line_numbers" | "line_numbers" => {
                // Toggle line numbers in the config
                let app = self.app_state.read().await;
                let config_dir = app.user_dir.clone();
                let current_setting = app.get_line_numbers_setting();
                drop(app);

                // Update the config file
                let mut config_manager = crate::config::ConfigManager::new(&config_dir);
                if config_manager.load().is_ok() {
                    // Toggle the setting
                    config_manager.get_config_mut().editor.show_line_numbers = !current_setting;
                    if let Err(e) = config_manager.save() {
                        let error_msg = format!("Error saving config: {}", e);
                        self.event_sender.send(AppEvent::ToastMessage {
                            message: error_msg.into(),
                            toast_type: "error".into(),
                        })?;
                    } else {
                        let status = if !current_setting {
                            "enabled"
                        } else {
                            "disabled"
                        };
                        let line_status_msg = format!("Line numbers {}", status);
                        self.event_sender.send(AppEvent::ToastMessage {
                            message: line_status_msg.into(),
                            toast_type: "info".into(),
                        })?;
                    }
                }
            }
            _ => {
                let unknown_cmd_msg = format!("Unknown command: {}", parts[0]);
                self.event_sender.send(AppEvent::StatusMessage {
                    message: unknown_cmd_msg.into(),
                })?;
            }
        }

        // Clear command input
        let mut app = self.app_state.write().await;
        app.command_input.clear();

        Ok(())
    }

    /// Handle opening a file
    async fn handle_open_file(&self, file_path: &str) -> Result<()> {
        let path = std::path::PathBuf::from(file_path);

        match crate::buffer::Buffer::from_path_async(path.clone()).await {
            Ok(buffer) => {
                let mut app = self.app_state.write().await;
                app.add_buffer(buffer);
                drop(app);

                let success_message = format!("Opened file: {}", file_path);
                let success_msg: Arc<str> = success_message.into();
                self.event_sender.send(AppEvent::ToastMessage {
                    message: success_msg.clone(),
                    toast_type: "success".into(),
                })?;
                self.event_sender.send(AppEvent::StatusMessage {
                    message: success_msg,
                })?;
            }
            Err(e) => {
                let error_message = format!("Error opening file: {}", e);
                let error_msg: Arc<str> = error_message.into();
                self.event_sender.send(AppEvent::ToastMessage {
                    message: error_msg.clone(),
                    toast_type: "error".into(),
                })?;
                self.event_sender
                    .send(AppEvent::StatusMessage { message: error_msg })?;
            }
        }

        Ok(())
    }
}

impl Clone for KeyboardHandler {
    fn clone(&self) -> Self {
        Self {
            app_state: self.app_state.clone(),
            event_sender: self.event_sender.clone(),
        }
    }
}
