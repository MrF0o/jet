use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Result};
use ratatui::{
    backend::CrosstermBackend,
    crossterm::event::{self, Event},
    Terminal,
};
use tokio::sync::RwLock;

use crate::buffer::Buffer;
use crate::events::EventBus;
use crate::handlers::{AppStateHandler, KeyboardHandler, MouseHandler};
use crate::input_system::InputSystem;
use crate::widgets::CursorManager;

/// Contains global state that needs to be shared
pub struct App {
    /// Whether the application is running
    pub running: bool,

    /// List of open buffers
    pub buffers: Vec<Buffer>,

    /// Currently active buffer index
    pub active_buffer: usize,

    /// Scroll position for the active editor
    pub scroll_offset: (usize, usize),

    /// Command mode state
    pub command_mode: CommandMode,

    /// Command input (for command mode)
    pub command_input: String,

    /// Message to display on status bar
    pub status_message: Option<String>,

    /// Directory where user extensions and config will be stored
    pub user_dir: PathBuf,

    /// Background task management for async operations
    pub background_tasks: BackgroundTasks,

    /// Toast notification manager
    pub toast_manager: crate::widgets::toast::ToastManager,

    /// Whether to show the command palette modal
    pub show_command_palette: bool,

    /// Cursor manager for handling multiple independent cursors
    pub cursor_manager: CursorManager,

    /// Status bar with slot-based system
    pub status_bar: crate::widgets::StatusBar,

    /// Mouse drag start position for text selection
    pub mouse_drag_start: Option<(usize, usize)>,
}

/// Command input modes
#[derive(Debug, PartialEq, Clone)]
pub enum CommandMode {
    /// Normal mode (no command input)
    Normal,

    /// Command palette mode
    Command,

    /// File search mode
    FileSearch,

    /// Text search mode
    TextSearch,
}

impl App {
    pub async fn new() -> Self {
        let user_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("editor");

        // Create user directory if it doesn't exist
        if !user_dir.exists() {
            if let Err(e) = tokio::fs::create_dir_all(&user_dir).await {
                eprintln!("Warning: Could not create user directory: {}", e);
            }
        }

        let mut app = Self {
            running: true,
            buffers: vec![Buffer::new()],
            active_buffer: 0,
            scroll_offset: (0, 0),
            command_mode: CommandMode::Normal,
            command_input: String::new(),
            status_message: None,
            user_dir,
            background_tasks: BackgroundTasks::default(),
            toast_manager: crate::widgets::toast::ToastManager::new(),
            show_command_palette: false,
            cursor_manager: CursorManager::new(),
            status_bar: crate::widgets::StatusBar::new(),
            mouse_drag_start: None,
        };
        
        app.init_status_bar();
        app
    }

    pub async fn with_file(file_path: &str) -> Result<Self> {
        let user_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("editor");

        // Create user directory if it doesn't exist
        if !user_dir.exists() {
            tokio::fs::create_dir_all(&user_dir).await?;
        }

        let buffer = Buffer::from_path_async(PathBuf::from(file_path))
            .await
            .map_err(|e| anyhow!("Failed to open file '{}': {}", file_path, e))?;

        let mut app = Self {
            running: true,
            buffers: vec![buffer],
            active_buffer: 0,
            scroll_offset: (0, 0),
            command_mode: CommandMode::Normal,
            command_input: String::new(),
            status_message: None,
            user_dir,
            background_tasks: BackgroundTasks::default(),
            toast_manager: crate::widgets::toast::ToastManager::new(),
            show_command_palette: false,
            cursor_manager: CursorManager::new(),
            status_bar: crate::widgets::StatusBar::new(),
            mouse_drag_start: None,
        };
        
        app.init_status_bar();
        Ok(app)
    }

    /// Run the application with the new event-driven architecture
    pub async fn run(&mut self, terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<bool> {
        // Create user config directory if it doesn't exist
        if !self.user_dir.exists() {
            tokio::fs::create_dir_all(&self.user_dir).await?;
        }

        // Create shared app state
        let app_state = Arc::new(RwLock::new(std::mem::take(self)));

        // Create event bus and input system
        let event_bus = EventBus::new();
        let input_system = InputSystem::new(event_bus.clone());

        // Create and subscribe event handlers
        let keyboard_handler = KeyboardHandler::new(app_state.clone(), input_system.event_sender());
        let mouse_handler = MouseHandler::new(app_state.clone(), input_system.event_sender());
        let app_state_handler = AppStateHandler::new(app_state.clone());

        keyboard_handler.subscribe(&event_bus).await?;
        mouse_handler.subscribe(&event_bus).await?;
        app_state_handler.subscribe(&event_bus).await?;

        // Start event processing in background
        let event_bus_clone = event_bus.clone();
        tokio::spawn(async move {
            if let Err(e) = event_bus_clone.start_processing().await {
                eprintln!("Event processing error: {}", e);
            }
        });

        // Target frame rate
        let frame_duration = Duration::from_millis(16);
        let mut last_frame = Instant::now();

        // Main event loop
        loop {
            let frame_start = Instant::now();

            // Check if app should quit
            {
                let app = app_state.read().await;
                if !app.running {
                    break;
                }
            }

            // Draw the UI - limit to target frame rate
            if frame_start.duration_since(last_frame) >= frame_duration {
                let mut app = app_state.write().await;
                if let Err(e) = terminal.draw(|f| app.render(f)) {
                    eprintln!("Rendering error: {}", e);
                    break;
                }
                drop(app); // Release lock immediately after drawing
                last_frame = frame_start;
            }

            // Handle events with timeout to maintain frame rate
            // Check for events without blocking
            if event::poll(Duration::from_millis(1))? {
                match event::read()? {
                    Event::Key(key) => {
                        if let Err(e) = input_system.handle_key_input(key) {
                            eprintln!("Error handling key input: {}", e);
                        }
                    }
                    Event::Mouse(mouse) => {
                        if let Err(e) = input_system.handle_mouse_input(mouse) {
                            eprintln!("Error handling mouse input: {}", e);
                        }
                    }
                    Event::Resize(_, _) => {
                        // Handle resize if needed
                    }
                    _ => {}
                }
            } else {
                tokio::time::sleep(Duration::from_millis(1)).await;
            }
        }

        match Arc::try_unwrap(app_state) {
            Ok(app_mutex) => {
                *self = app_mutex.into_inner();
            }
            Err(app_state_arc) => {
                // Fallback if there are still references (shouldn't happen in normal operation)
                eprintln!(
                    "Warning: App state still has multiple references, using expensive clone fallback"
                );
                let app_guard = app_state_arc.read().await;
                *self = app_guard.clone();
            }
        }

        Ok(true)
    }

    /// Get the currently active buffer, if any
    pub fn get_active_buffer(&self) -> Option<&Buffer> {
        self.buffers.get(self.active_buffer)
    }

    /// Get a mutable reference to the currently active buffer, if any
    pub fn get_active_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.active_buffer)
    }

    /// Switch to a different buffer by index
    pub fn switch_to_buffer(&mut self, index: usize) -> bool {
        if index < self.buffers.len() {
            self.active_buffer = index;
            // Reset scroll when switching buffers
            self.scroll_offset = (0, 0);
            true
        } else {
            false
        }
    }

    /// Close the current buffer
    pub fn close_current_buffer(&mut self) -> bool {
        if self.buffers.len() <= 1 {
            // Don't close the last buffer
            return false;
        }

        self.buffers.remove(self.active_buffer);

        // Adjust active buffer index if necessary
        if self.active_buffer >= self.buffers.len() {
            self.active_buffer = self.buffers.len() - 1;
        }

        // Reset scroll when closing buffer
        self.scroll_offset = (0, 0);
        true
    }

    /// Add a new buffer
    pub fn add_buffer(&mut self, buffer: Buffer) -> usize {
        self.buffers.push(buffer);
        let new_index = self.buffers.len() - 1;
        self.active_buffer = new_index;
        self.scroll_offset = (0, 0);
        new_index
    }

    /// Set a status message with automatic timeout
    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    /// Clear the status message
    pub fn clear_status_message(&mut self) {
        self.status_message = None;
    }

    /// Check if any buffers have unsaved changes
    pub fn has_unsaved_changes(&self) -> bool {
        self.buffers.iter().any(|buffer| buffer.is_dirty())
    }

    /// Get the number of open buffers
    pub fn buffer_count(&self) -> usize {
        self.buffers.len()
    }

    /// Initialize the status bar with default slots
    pub fn init_status_bar(&mut self) {
        use crate::widgets::{SlotAlignment, StatusSlot};
        use ratatui::style::{Color, Style};

        // File info slot (left side, high priority)
        let file_slot = StatusSlot::new("file", "")
            .with_alignment(SlotAlignment::Left)
            .with_priority(100)
            .with_style(Style::default().fg(Color::White).bg(Color::LightBlue));
        self.status_bar.set_slot(file_slot);

        // Cursor position slot (left side, medium priority)
        let cursor_slot = StatusSlot::new("cursor", "")
            .with_alignment(SlotAlignment::Left)
            .with_priority(90)
            .with_style(Style::default().fg(Color::White).bg(Color::LightBlue));
        self.status_bar.set_slot(cursor_slot);

        // Modified status slot (left side, medium priority)
        let modified_slot = StatusSlot::new("modified", "")
            .with_alignment(SlotAlignment::Left)
            .with_priority(80)
            .with_style(Style::default().fg(Color::White).bg(Color::LightBlue));
        self.status_bar.set_slot(modified_slot);

        // Selection info slot (center, when applicable)
        let selection_slot = StatusSlot::new("selection", "")
            .with_alignment(SlotAlignment::Center)
            .with_priority(70)
            .with_style(Style::default().fg(Color::Black).bg(Color::Yellow))
            .with_visibility(false); // Hidden by default
        self.status_bar.set_slot(selection_slot);

        // Mode indicator slot (right side, high priority)
        let mode_slot = StatusSlot::new("mode", "NORMAL")
            .with_alignment(SlotAlignment::Right)
            .with_priority(100)
            .with_style(Style::default().fg(Color::White).bg(Color::DarkGray));
        self.status_bar.set_slot(mode_slot);

        // Buffer count slot (right side, low priority)
        let buffer_count_slot = StatusSlot::new("buffer_count", "")
            .with_alignment(SlotAlignment::Right)
            .with_priority(60)
            .with_style(Style::default().fg(Color::Gray).bg(Color::LightBlue));
        self.status_bar.set_slot(buffer_count_slot);
    }

    /// Update status bar slots with current application state
    pub fn update_status_bar(&mut self) {
        if let Some(buffer) = self.buffers.get(self.active_buffer) {
            let (row, col) = buffer.cursor_pos;

            // Update file info
            self.status_bar.update_slot_content("file", &buffer.name);

            // Update cursor position
            let cursor_info = format!("Ln {}, Col {}", row + 1, col + 1);
            self.status_bar.update_slot_content("cursor", cursor_info);

            // Update modified status
            let modified_text = if buffer.modified { "Unsaved" } else { "Saved" };
            self.status_bar.update_slot_content("modified", modified_text);

            // Update selection info if there's a selection
            if let Some(selected_text) = buffer.get_selected_text() {
                let char_count = selected_text.len();
                let line_count = selected_text.matches('\n').count() + 1;
                let selection_info = if line_count > 1 {
                    format!("Selection: {} lines, {} chars", line_count, char_count)
                } else {
                    format!("Selection: {} chars", char_count)
                };
                self.status_bar.update_slot_content("selection", selection_info);
                self.status_bar.show_slot("selection");
            } else {
                self.status_bar.hide_slot("selection");
            }

            // Update mode indicator
            let mode_text = match self.command_mode {
                CommandMode::Normal => "NORMAL",
                CommandMode::Command => "COMMAND",
                CommandMode::FileSearch => "FILE SEARCH",
                CommandMode::TextSearch => "TEXT SEARCH",
            };
            self.status_bar.update_slot_content("mode", mode_text);

            // Update buffer count
            let buffer_info = format!("Buffer {}/{}", self.active_buffer + 1, self.buffers.len());
            self.status_bar.update_slot_content("buffer_count", buffer_info);
        }
    }
}

// Make App cloneable for the event system
// WARNING: This clone is expensive and should be avoided in hot paths!
// It clones all buffers and their content. Only used as a fallback when
// Arc::try_unwrap fails (which should be rare in normal operation).
// The main UI render path now uses references to avoid cloning.
impl Clone for App {
    fn clone(&self) -> Self {
        let mut app = Self {
            running: self.running,
            buffers: self.buffers.clone(),
            active_buffer: self.active_buffer,
            scroll_offset: self.scroll_offset,
            command_mode: self.command_mode.clone(),
            command_input: self.command_input.clone(),
            status_message: self.status_message.clone(),
            user_dir: self.user_dir.clone(),
            background_tasks: BackgroundTasks::default(), // Don't clone background tasks
            toast_manager: crate::widgets::toast::ToastManager::new(), // Create new instance
            show_command_palette: self.show_command_palette,
            cursor_manager: CursorManager::new(), // Create new instance
            status_bar: crate::widgets::StatusBar::new(), // Create new instance
            mouse_drag_start: self.mouse_drag_start,
        };
        
        app.init_status_bar();
        app
    }
}

impl Default for App {
    fn default() -> Self {
        let mut app = Self {
            running: true,
            buffers: vec![Buffer::new()],
            active_buffer: 0,
            scroll_offset: (0, 0),
            command_mode: CommandMode::Normal,
            command_input: String::new(),
            status_message: None,
            user_dir: PathBuf::from("."),
            background_tasks: BackgroundTasks::default(),
            toast_manager: crate::widgets::toast::ToastManager::new(),
            show_command_palette: false,
            cursor_manager: CursorManager::new(),
            status_bar: crate::widgets::StatusBar::new(),
            mouse_drag_start: None,
        };
        
        app.init_status_bar();
        app
    }
}

/// Background task management
#[derive(Default)]
pub struct BackgroundTasks {
    // TODO: This would contain task handles for background operations
}
