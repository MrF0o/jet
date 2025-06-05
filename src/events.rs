use anyhow::Result;
use ratatui::crossterm::event::{KeyEvent, MouseEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

/// All possible events in the application
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Keyboard input events
    KeyInput(KeyEvent),

    /// Mouse input events
    MouseInput(MouseEvent),

    /// Buffer-related events
    BufferChanged {
        buffer_id: usize,
        content: Arc<str>,
    },
    BufferCursorMoved {
        buffer_id: usize,
        row: usize,
        col: usize,
    },
    BufferSelectionChanged {
        buffer_id: usize,
        start: Option<(usize, usize)>,
        end: Option<(usize, usize)>,
    },

    /// UI events
    ModeChanged {
        new_mode: Arc<str>,
    },
    StatusMessage {
        message: Arc<str>,
    },
    ToastMessage {
        message: Arc<str>,
        toast_type: Arc<str>,
    },
    ShowCommandPalette,
    HideCommandPalette,

    /// Cursor events
    CursorShow {
        context: Arc<str>,
    },
    CursorHide {
        context: Arc<str>,
    },
    CursorMove {
        context: Arc<str>,
        row: usize,
        col: usize,
    },

    /// Application lifecycle
    Quit,
    Refresh,
}

/// Event priority levels for ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
pub enum EventPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Critical = 3,
}

/// Enhanced event with priority and metadata
#[derive(Debug, Clone)]
pub struct PrioritizedEvent {
    pub event: AppEvent,
    pub priority: EventPriority,
    pub timestamp: std::time::Instant,
}

impl PrioritizedEvent {
    pub fn new(event: AppEvent) -> Self {
        Self {
            event,
            priority: EventPriority::Normal,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn with_priority(event: AppEvent, priority: EventPriority) -> Self {
        Self {
            event,
            priority,
            timestamp: std::time::Instant::now(),
        }
    }
}

/// Event handler function type
pub type EventHandler = Arc<dyn Fn(&AppEvent) -> Result<()> + Send + Sync>;

/// Async event handler function type
pub type AsyncEventHandler = Arc<
    dyn Fn(AppEvent) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>
        + Send
        + Sync,
>;

/// Event bus for publishing and subscribing to events
#[derive(Clone)]
pub struct EventBus {
    /// Sync event handlers
    handlers: Arc<RwLock<HashMap<String, Vec<EventHandler>>>>,

    /// Async event handlers
    async_handlers: Arc<RwLock<HashMap<String, Vec<AsyncEventHandler>>>>,

    /// Channel for sending events
    sender: mpsc::UnboundedSender<AppEvent>,

    /// Channel for receiving events
    receiver: Arc<RwLock<Option<mpsc::UnboundedReceiver<AppEvent>>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            async_handlers: Arc::new(RwLock::new(HashMap::new())),
            sender,
            receiver: Arc::new(RwLock::new(Some(receiver))),
        }
    }

    /// Get a sender for publishing events
    pub fn sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.sender.clone()
    }

    /// Subscribe to events with a sync handler
    pub async fn subscribe<F>(&self, event_type: &str, handler: F)
    where
        F: Fn(&AppEvent) -> Result<()> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.write().await;
        handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(handler));
    }

    /// Subscribe to events with an async handler
    pub async fn subscribe_async<F, Fut>(&self, event_type: &str, handler: F)
    where
        F: Fn(AppEvent) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        let mut async_handlers = self.async_handlers.write().await;
        async_handlers
            .entry(event_type.to_string())
            .or_insert_with(Vec::new)
            .push(Arc::new(move |event| Box::pin(handler(event))));
    }

    /// Publish an event
    pub fn publish(&self, event: AppEvent) -> Result<()> {
        self.sender.send(event)?;
        Ok(())
    }

    /// Start processing events (should be called once in a background task)
    pub async fn start_processing(&self) -> Result<()> {
        let mut receiver = {
            let mut receiver_guard = self.receiver.write().await;
            receiver_guard
                .take()
                .ok_or_else(|| anyhow::anyhow!("Event processor already started"))?
        };

        while let Some(event) = receiver.recv().await {
            self.handle_event(event).await;
        }

        Ok(())
    }

    /// Handle a single event by calling all registered handlers
    async fn handle_event(&self, event: AppEvent) {
        let event_type = self.get_event_type(&event);

        // Handle sync handlers
        {
            let handlers = self.handlers.read().await;
            if let Some(event_handlers) = handlers.get(event_type) {
                for handler in event_handlers {
                    if let Err(e) = handler(&event) {
                        eprintln!("Error in sync event handler for {}: {}", event_type, e);
                    }
                }
            }
        }

        // Handle async handlers
        {
            let async_handlers = self.async_handlers.read().await;
            if let Some(event_handlers) = async_handlers.get(event_type) {
                for handler in event_handlers {
                    if let Err(e) = handler(event.clone()).await {
                        eprintln!("Error in async event handler for {}: {}", event_type, e);
                    }
                }
            }
        }
    }

    /// Get the event type string for routing
    fn get_event_type(&self, event: &AppEvent) -> &'static str {
        match event {
            AppEvent::KeyInput(_) => "key_input",
            AppEvent::MouseInput(_) => "mouse_input",
            AppEvent::BufferChanged { .. } => "buffer_changed",
            AppEvent::BufferCursorMoved { .. } => "buffer_cursor_moved",
            AppEvent::BufferSelectionChanged { .. } => "buffer_selection_changed",
            AppEvent::ModeChanged { .. } => "mode_changed",
            AppEvent::StatusMessage { .. } => "status_message",
            AppEvent::ToastMessage { .. } => "toast_message",
            AppEvent::ShowCommandPalette => "show_command_palette",
            AppEvent::HideCommandPalette => "hide_command_palette",
            AppEvent::CursorShow { .. } => "cursor_show",
            AppEvent::CursorHide { .. } => "cursor_hide",
            AppEvent::CursorMove { .. } => "cursor_move",
            AppEvent::Quit => "quit",
            AppEvent::Refresh => "refresh",
        }
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
