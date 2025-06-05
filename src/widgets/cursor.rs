use ratatui::{prelude::*, widgets::StatefulWidget};

/// A cursor widget that can render and manage cursor state independently
#[derive(Debug, Clone)]
pub struct Cursor {
    /// The visual position of the cursor (x, y) within its widget area
    pub position: Position,
    /// Whether this cursor is active/visible
    pub active: bool,
    /// The style of the cursor
    pub style: Style,
    /// Cursor identifier for tracking multiple cursors
    pub id: String,
}

/// State for the cursor widget
#[derive(Debug, Clone)]
pub struct CursorState {
    /// The current position of the cursor within the widget area
    pub position: Position,
    /// Whether the cursor is currently visible
    pub visible: bool,
    /// Last update time for blinking
    pub last_blink: std::time::Instant,
    /// Whether cursor is in blink-on phase
    pub blink_on: bool,
    /// Last activity time (typing, cursor movement, etc.)
    pub last_activity: std::time::Instant,
    /// Duration to keep cursor solid after activity before starting to blink
    pub activity_timeout: std::time::Duration,
}

impl Default for CursorState {
    fn default() -> Self {
        Self {
            position: Position::new(0, 0),
            visible: false,
            last_blink: std::time::Instant::now(),
            blink_on: true,
            last_activity: std::time::Instant::now(),
            activity_timeout: std::time::Duration::from_millis(1000), // 1 second before blinking starts
        }
    }
}

impl Cursor {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            position: Position::new(0, 0),
            active: true,
            style: Style::default().bg(Color::White).fg(Color::Black),
            id: id.into(),
        }
    }

    pub fn with_position(mut self, x: u16, y: u16) -> Self {
        self.position = Position::new(x, y);
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl StatefulWidget for Cursor {
    type State = CursorState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if !self.active || !state.visible {
            return;
        }

        let now = std::time::Instant::now();

        // Check if we're still in the activity period (cursor should be solid)
        let in_activity_period = now.duration_since(state.last_activity) < state.activity_timeout;

        // Update blink state only if we're past the activity period
        let should_show_cursor = if in_activity_period {
            // During activity period, always show cursor (no blinking)
            true
        } else {
            // After activity period, start blinking
            if now.duration_since(state.last_blink) > std::time::Duration::from_millis(500) {
                state.blink_on = !state.blink_on;
                state.last_blink = now;
            }
            state.blink_on
        };

        // Only render if we should show the cursor
        if should_show_cursor {
            let cursor_x = self.position.x;
            let cursor_y = self.position.y;

            // Ensure cursor is within bounds
            if cursor_x < area.width && cursor_y < area.height {
                if let Some(cell) = buf.cell_mut(Position::new(cursor_x, cursor_y)) {
                    // Set cursor by changing the background color of the cell
                    // This works for any character including spaces and empty cells
                    cell.set_bg(Color::White);
                    cell.set_fg(Color::Black);
                }
            }
        }

        // Update state
        state.position = self.position;
    }
}

/// Manager for handling multiple cursors in different contexts
#[derive(Debug, Default)]
pub struct CursorManager {
    /// Map of cursor contexts to their states
    cursors: std::collections::HashMap<String, CursorState>,
    /// Currently active cursor context
    active_context: Option<String>,
}

impl CursorManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Get or create a cursor state for a given context
    pub fn get_or_create_cursor(&mut self, context: &str) -> &mut CursorState {
        self.cursors
            .entry(context.to_string())
            .or_insert_with(CursorState::default)
    }

    /// Set the active cursor context - ONLY ONE CURSOR ACTIVE AT A TIME
    pub fn set_active_context(&mut self, context: &str) {
        // Hide ALL cursors first
        for (_, cursor_state) in self.cursors.iter_mut() {
            cursor_state.visible = false;
        }

        // Set new active context
        self.active_context = Some(context.to_string());

        // Show ONLY the active cursor
        let cursor_state = self.get_or_create_cursor(context);
        cursor_state.visible = true;
    }

    /// Get the active cursor context
    pub fn get_active_context(&self) -> Option<&str> {
        self.active_context.as_deref()
    }

    /// Update cursor position for a specific context
    pub fn update_cursor_position(&mut self, context: &str, x: u16, y: u16) {
        // ONLY update if this is the active context
        if self.active_context.as_deref() == Some(context) {
            let cursor_state = self.get_or_create_cursor(context);

            // Check if position actually changed (cursor movement activity)
            let position_changed = cursor_state.position != Position::new(x, y);

            cursor_state.position = Position::new(x, y);
            cursor_state.visible = true;

            // If position changed, treat as activity
            if position_changed {
                cursor_state.last_activity = std::time::Instant::now();
                cursor_state.blink_on = true; // Show cursor immediately
            }
        }
    }

    /// Hide cursor for a specific context
    pub fn hide_cursor(&mut self, context: &str) {
        if let Some(cursor_state) = self.cursors.get_mut(context) {
            cursor_state.visible = false;
        }

        if self.active_context.as_deref() == Some(context) {
            self.active_context = None;
        }
    }

    /// Show cursor for a specific context
    pub fn show_cursor(&mut self, context: &str) {
        // ONLY show if this is the active context
        if self.active_context.as_deref() == Some(context) {
            let cursor_state = self.get_or_create_cursor(context);
            cursor_state.visible = true;
        }
    }

    /// Get the current cursor position for a context, if visible
    pub fn get_cursor_position(&self, context: &str) -> Option<Position> {
        self.cursors.get(context).and_then(|state| {
            if state.visible && self.active_context.as_deref() == Some(context) {
                Some(state.position)
            } else {
                None
            }
        })
    }

    /// Get a mutable reference to a specific cursor state
    pub fn get_cursor_state_mut(&mut self, context: &str) -> Option<&mut CursorState> {
        if self.active_context.as_deref() == Some(context) {
            self.cursors.get_mut(context)
        } else {
            None
        }
    }

    /// Notify about user activity (typing, cursor movement, etc.) for a specific context
    pub fn notify_activity(&mut self, context: &str) {
        if let Some(cursor_state) = self.cursors.get_mut(context) {
            cursor_state.last_activity = std::time::Instant::now();
            cursor_state.blink_on = true; // Ensure cursor is visible immediately
        }
    }

    /// Notify about user activity for the currently active context
    pub fn notify_activity_for_active(&mut self) {
        if let Some(active_context) = self.active_context.clone() {
            self.notify_activity(&active_context);
        }
    }

    /// Clear all cursors
    pub fn clear_all(&mut self) {
        self.cursors.clear();
        self.active_context = None;
    }
}

/// Helper trait for widgets that need cursor support
pub trait CursorSupport {
    /// Calculate the cursor position within the widget based on logical position
    fn calculate_cursor_position(&self, logical_pos: (usize, usize), area: Rect) -> Position;

    /// Get the cursor context identifier for this widget
    fn get_cursor_context(&self) -> &str;
}
