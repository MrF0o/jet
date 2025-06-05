use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget},
};
use std::time::{Duration, Instant};

/// Type of toast notification
#[derive(Debug, Clone, PartialEq)]
pub enum ToastType {
    Info,
    Success,
    Warning,
    Error,
}

impl ToastType {
    fn color(&self) -> Color {
        match self {
            ToastType::Info => Color::Cyan,
            ToastType::Success => Color::Green,
            ToastType::Warning => Color::Yellow,
            ToastType::Error => Color::Red,
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            ToastType::Info => "ℹ",
            ToastType::Success => "✓",
            ToastType::Warning => "⚠",
            ToastType::Error => "✗",
        }
    }
}

/// A single toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: Instant,
    pub duration: Duration,
}

impl Toast {
    pub fn new(message: String, toast_type: ToastType) -> Self {
        Self {
            message,
            toast_type,
            created_at: Instant::now(),
            duration: Duration::from_secs(3), // Default 3 seconds
        }
    }

    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.duration
    }

    pub fn remaining_time(&self) -> Duration {
        self.duration.saturating_sub(self.created_at.elapsed())
    }

    /// Get the progress of the toast (0.0 = just created, 1.0 = expired)
    pub fn progress(&self) -> f32 {
        let elapsed = self.created_at.elapsed().as_secs_f32();
        let total = self.duration.as_secs_f32();
        (elapsed / total).min(1.0)
    }
}

/// Toast notification manager and renderer
pub struct ToastManager {
    toasts: Vec<Toast>,
    max_toasts: usize,
}

impl ToastManager {
    pub fn new() -> Self {
        Self {
            toasts: Vec::new(),
            max_toasts: 5,
        }
    }

    pub fn add_toast(&mut self, toast: Toast) {
        self.toasts.push(toast);

        // Remove oldest toasts if we exceed the maximum
        while self.toasts.len() > self.max_toasts {
            self.toasts.remove(0);
        }
    }

    pub fn add_info(&mut self, message: String) {
        self.add_toast(Toast::new(message, ToastType::Info));
    }

    pub fn add_success(&mut self, message: String) {
        self.add_toast(Toast::new(message, ToastType::Success));
    }

    pub fn add_warning(&mut self, message: String) {
        self.add_toast(Toast::new(message, ToastType::Warning));
    }

    pub fn add_error(&mut self, message: String) {
        self.add_toast(Toast::new(message, ToastType::Error));
    }

    pub fn update(&mut self) {
        // Remove expired toasts
        self.toasts.retain(|toast| !toast.is_expired());
    }

    pub fn has_active_toasts(&self) -> bool {
        !self.toasts.is_empty()
    }

    pub fn render(&self, area: Rect, buf: &mut TuiBuffer) {
        if self.toasts.is_empty() {
            return;
        }

        // Calculate toast area (top-right corner)
        let toast_width = 40.min(area.width / 3);
        let toast_height = (self.toasts.len() as u16 * 3).min(area.height / 2); // 3 lines per toast

        let toast_area = Rect {
            x: area.width.saturating_sub(toast_width + 2),
            y: 2,
            width: toast_width,
            height: toast_height,
        };

        // Render each toast
        for (i, toast) in self.toasts.iter().enumerate() {
            let y_offset = i as u16 * 3;
            if y_offset >= toast_area.height {
                break;
            }

            let individual_toast_area = Rect {
                x: toast_area.x,
                y: toast_area.y + y_offset,
                width: toast_area.width,
                height: 3.min(toast_area.height - y_offset),
            };

            self.render_single_toast(toast, individual_toast_area, buf);
        }
    }

    fn render_single_toast(&self, toast: &Toast, area: Rect, buf: &mut TuiBuffer) {
        // Create a subtle animation effect based on progress
        let progress = toast.progress();
        let alpha = if progress > 0.8 {
            // Fade out in the last 20% of duration
            ((1.0 - progress) / 0.2).min(1.0)
        } else {
            1.0
        };

        // Choose colors based on type and alpha
        let primary_color = toast.toast_type.color();
        let border_color = if alpha < 0.5 {
            Color::DarkGray
        } else {
            primary_color
        };

        // Clear the area first
        Clear.render(area, buf);

        // Create the toast block with rounded appearance
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Rgb(30, 30, 30))); // Dark background

        let inner_area = block.inner(area);
        block.render(area, buf);

        // Create the toast content
        let icon = toast.toast_type.icon();
        let message = if toast.message.len() > (inner_area.width as usize).saturating_sub(4) {
            // Truncate long messages with pre-allocated string
            let max_len = (inner_area.width as usize).saturating_sub(7); // Leave space for "..."
            let truncate_len = max_len.min(toast.message.len());
            let mut truncated = String::with_capacity(truncate_len + 3);
            truncated.push_str(&toast.message[..truncate_len]);
            truncated.push_str("...");
            truncated
        } else {
            toast.message.clone()
        };

        let mut icon_text = String::with_capacity(icon.len() + 1);
        icon_text.push_str(icon);
        icon_text.push(' ');

        let content = Line::from(vec![
            Span::styled(
                icon_text,
                Style::default()
                    .fg(primary_color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(message, Style::default().fg(Color::White)),
        ]);

        // Progress bar at the bottom
        let progress_width = ((1.0 - progress) * inner_area.width as f32) as u16;

        let progress_line = if progress_width > 0 {
            Line::from(vec![
                Span::styled(
                    "█".repeat(progress_width as usize),
                    Style::default().fg(primary_color),
                ),
                Span::styled(
                    "░".repeat((inner_area.width - progress_width) as usize),
                    Style::default().fg(Color::DarkGray),
                ),
            ])
        } else {
            Line::from("") // Empty line when expired
        };

        // Render content and progress bar
        if inner_area.height >= 2 {
            let content_paragraph = Paragraph::new(content);
            let content_area = Rect {
                x: inner_area.x,
                y: inner_area.y,
                width: inner_area.width,
                height: 1,
            };
            content_paragraph.render(content_area, buf);

            let progress_paragraph = Paragraph::new(progress_line);
            let progress_area = Rect {
                x: inner_area.x,
                y: inner_area.y + 1,
                width: inner_area.width,
                height: 1,
            };
            progress_paragraph.render(progress_area, buf);
        } else {
            // Just render content if height is too small
            let content_paragraph = Paragraph::new(content);
            content_paragraph.render(inner_area, buf);
        }
    }
}

impl Default for ToastManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenient widget wrapper for rendering toasts
pub struct ToastWidget<'a> {
    manager: &'a ToastManager,
}

impl<'a> ToastWidget<'a> {
    pub fn new(manager: &'a ToastManager) -> Self {
        Self { manager }
    }
}

impl<'a> Widget for ToastWidget<'a> {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        self.manager.render(area, buf);
    }
}
