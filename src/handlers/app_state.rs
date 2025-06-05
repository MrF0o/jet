/// Application state handlers that respond to events
use crate::events::{AppEvent, EventBus};
use crate::{App, CommandMode};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// App state handler that manages application state in response to events
pub struct AppStateHandler {
    app_state: Arc<RwLock<App>>,
}

impl AppStateHandler {
    /// Create a new app state handler
    pub fn new(app_state: Arc<RwLock<App>>) -> Self {
        Self { app_state }
    }

    /// Subscribe to all relevant events
    pub async fn subscribe(&self, event_bus: &EventBus) -> Result<()> {
        let handler = AppStateHandler::new(self.app_state.clone());

        // Subscribe to mode changes
        event_bus
            .subscribe_async("mode_changed", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_mode_changed(event).await }
                }
            })
            .await;

        // Subscribe to status messages
        event_bus
            .subscribe_async("status_message", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_status_message(event).await }
                }
            })
            .await;

        // Subscribe to toast messages
        event_bus
            .subscribe_async("toast_message", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_toast_message(event).await }
                }
            })
            .await;

        // Subscribe to command palette events
        event_bus
            .subscribe_async("show_command_palette", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_show_command_palette(event).await }
                }
            })
            .await;

        event_bus
            .subscribe_async("hide_command_palette", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_hide_command_palette(event).await }
                }
            })
            .await;
        // Subscribe to quit events
        event_bus
            .subscribe_async("quit", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_quit(event).await }
                }
            })
            .await;

        // Subscribe to buffer cursor moved events
        event_bus
            .subscribe_async("buffer_cursor_moved", {
                let handler = handler.clone();
                move |event| {
                    let handler = handler.clone();
                    async move { handler.handle_buffer_cursor_moved(event).await }
                }
            })
            .await;

        Ok(())
    }

    /// Handle mode change events
    async fn handle_mode_changed(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::ModeChanged { new_mode } = event {
            let mut app = self.app_state.write().await;

            app.command_mode = match new_mode.as_ref() {
                "normal" => CommandMode::Normal,
                "command" => CommandMode::Command,
                "file_search" => CommandMode::FileSearch,
                "text_search" => CommandMode::TextSearch,
                _ => CommandMode::Normal,
            };

            // Clear command input when switching to normal mode
            if app.command_mode == CommandMode::Normal {
                app.command_input.clear();
            }
        }

        Ok(())
    }

    /// Handle status message events
    async fn handle_status_message(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::StatusMessage { message } = event {
            let mut app = self.app_state.write().await;
            app.status_message = Some(message.to_string());

            // Also add the message as a toast notification
            use crate::widgets::toast::{Toast, ToastType};
            let message_str = message.as_ref();
            let message_lower = message_str.to_lowercase();

            let toast = if message_lower.contains("error") {
                Toast::new(message_str.to_string(), ToastType::Error)
            } else if message_lower.contains("success") || message_lower.contains("saved") {
                Toast::new(message_str.to_string(), ToastType::Success)
            } else if message_lower.contains("warning") {
                Toast::new(message_str.to_string(), ToastType::Warning)
            } else {
                Toast::new(message_str.to_string(), ToastType::Info)
            };

            app.toast_manager.add_toast(toast);
        }

        Ok(())
    }

    /// Handle show command palette events
    async fn handle_show_command_palette(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::ShowCommandPalette = event {
            let mut app = self.app_state.write().await;
            app.show_command_palette = true;
            app.command_input.clear();
        }

        Ok(())
    }

    /// Handle hide command palette events
    async fn handle_hide_command_palette(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::HideCommandPalette = event {
            let mut app = self.app_state.write().await;
            app.show_command_palette = false;
            app.command_input.clear();
        }

        Ok(())
    }

    /// Handle quit events
    async fn handle_quit(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::Quit = event {
            let mut app = self.app_state.write().await;
            app.running = false;
        }

        Ok(())
    }

    /// Handle buffer cursor moved events - ensure cursor is visible when moved programmatically
    async fn handle_buffer_cursor_moved(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::BufferCursorMoved { .. } = event {
            // When cursor is moved programmatically (via keyboard), ensure it's visible
            // This is different from manual scrolling which shouldn't affect cursor visibility

            // We need the terminal size to call ensure_cursor_visible
            // For now, we'll use a reasonable default and improve this later
            use ratatui::crossterm::terminal;
            use ratatui::prelude::Rect;

            // Get terminal size - use a reasonable default if not available
            let terminal_size = terminal::size().unwrap_or((80, 24));
            let editor_area = Rect::new(0, 0, terminal_size.0, terminal_size.1.saturating_sub(2)); // Leave space for status

            let mut app = self.app_state.write().await;
            app.ensure_cursor_visible(editor_area);
        }

        Ok(())
    }

    /// Handle toast message events
    async fn handle_toast_message(&self, event: AppEvent) -> Result<()> {
        if let AppEvent::ToastMessage {
            message,
            toast_type,
        } = event
        {
            let mut app = self.app_state.write().await;

            use crate::widgets::toast::{Toast, ToastType};
            let toast_type = match toast_type.as_ref() {
                "error" => ToastType::Error,
                "success" => ToastType::Success,
                "warning" => ToastType::Warning,
                _ => ToastType::Info,
            };

            let toast = Toast::new(message.to_string(), toast_type);
            app.toast_manager.add_toast(toast);
        }

        Ok(())
    }
}

impl Clone for AppStateHandler {
    fn clone(&self) -> Self {
        Self {
            app_state: self.app_state.clone(),
        }
    }
}
