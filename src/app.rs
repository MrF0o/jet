use std::io::Stdout;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::event::{self, Event},
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

        Self {
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
            mouse_drag_start: None,
        }
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

        Ok(Self {
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
            mouse_drag_start: None,
        })
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
}

// Make App cloneable for the event system
// WARNING: This clone is expensive and should be avoided in hot paths!
// It clones all buffers and their content. Only used as a fallback when
// Arc::try_unwrap fails (which should be rare in normal operation).
// The main UI render path now uses references to avoid cloning.
impl Clone for App {
    fn clone(&self) -> Self {
        Self {
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
            mouse_drag_start: self.mouse_drag_start,
        }
    }
}

impl Default for App {
    fn default() -> Self {
        Self {
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
            mouse_drag_start: None,
        }
    }
}

/// Background task management
#[derive(Default)]
pub struct BackgroundTasks {
    // TODO: This would contain task handles for background operations
}
