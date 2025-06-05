use crate::events::{AppEvent, EventBus};
use anyhow::{Context, Result};
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use tokio::sync::mpsc;

/// Input system that handles raw input and publishes events
pub struct InputSystem {
    event_bus: EventBus,
}

impl InputSystem {
    /// Create a new input system
    pub fn new(event_bus: EventBus) -> Self {
        Self { event_bus }
    }

    /// Handle keyboard input by publishing a key event
    pub fn handle_key_input(&self, key: KeyEvent) -> Result<()> {
        self.event_bus
            .publish(AppEvent::KeyInput(key))
            .context("Failed to publish key input event")
    }

    /// Handle mouse input by publishing a mouse event
    pub fn handle_mouse_input(&self, mouse: MouseEvent) -> Result<()> {
        self.event_bus
            .publish(AppEvent::MouseInput(mouse))
            .context("Failed to publish mouse input event")
    }

    /// Get the event bus sender for direct event publishing
    pub fn event_sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.event_bus.sender()
    }
}
