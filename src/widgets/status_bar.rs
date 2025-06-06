use ratatui::{
    buffer::Buffer as TuiBuffer,
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::collections::HashMap;

/// Represents the alignment of a status bar slot
#[derive(Debug, Clone, PartialEq)]
pub enum SlotAlignment {
    Left,
    Center,
    Right,
}

/// Represents a single slot in the status bar
#[derive(Debug, Clone)]
pub struct StatusSlot {
    pub id: String,
    pub content: String,
    pub alignment: SlotAlignment,
    pub priority: u8, // Higher priority = shown first within alignment group
    pub style: Style,
    pub visible: bool,
    pub min_width: Option<u16>,
    pub max_width: Option<u16>,
}

impl StatusSlot {
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
            alignment: SlotAlignment::Left,
            priority: 50, // Default medium priority
            style: Style::default().fg(Color::White).bg(Color::LightBlue),
            visible: true,
            min_width: None,
            max_width: None,
        }
    }

    pub fn with_alignment(mut self, alignment: SlotAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }

    pub fn with_visibility(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub fn with_width_constraints(
        mut self,
        min_width: Option<u16>,
        max_width: Option<u16>,
    ) -> Self {
        self.min_width = min_width;
        self.max_width = max_width;
        self
    }
}

/// Status bar widget with slot-based system similar to VS Code
#[derive(Clone)]
pub struct StatusBar {
    slots: HashMap<String, StatusSlot>,
    background_style: Style,
    separator: String,
    show_separators: bool,
}

impl Default for StatusBar {
    fn default() -> Self {
        Self::new()
    }
}

impl StatusBar {
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
            background_style: Style::default().bg(Color::LightBlue).fg(Color::White),
            separator: " | ".to_string(),
            show_separators: true,
        }
    }

    /// Add or update a slot in the status bar
    pub fn set_slot(&mut self, slot: StatusSlot) {
        self.slots.insert(slot.id.clone(), slot);
    }

    /// Remove a slot from the status bar
    pub fn remove_slot(&mut self, id: &str) {
        self.slots.remove(id);
    }

    /// Get a mutable reference to a slot (for updating content)
    pub fn get_slot_mut(&mut self, id: &str) -> Option<&mut StatusSlot> {
        self.slots.get_mut(id)
    }

    /// Hide a slot without removing it
    pub fn hide_slot(&mut self, id: &str) {
        if let Some(slot) = self.slots.get_mut(id) {
            slot.visible = false;
        }
    }

    /// Show a previously hidden slot
    pub fn show_slot(&mut self, id: &str) {
        if let Some(slot) = self.slots.get_mut(id) {
            slot.visible = true;
        }
    }

    /// Update the content of a slot
    pub fn update_slot_content(&mut self, id: &str, content: impl Into<String>) {
        if let Some(slot) = self.slots.get_mut(id) {
            slot.content = content.into();
        }
    }

    /// Set the background style for the entire status bar
    pub fn with_background_style(mut self, style: Style) -> Self {
        self.background_style = style;
        self
    }

    /// Set the separator between slots
    pub fn with_separator(mut self, separator: impl Into<String>) -> Self {
        self.separator = separator.into();
        self
    }

    /// Enable or disable separators between slots
    pub fn with_separators(mut self, show_separators: bool) -> Self {
        self.show_separators = show_separators;
        self
    }

    /// Get all visible slots grouped by alignment and sorted by priority
    fn get_organized_slots(&self) -> (Vec<&StatusSlot>, Vec<&StatusSlot>, Vec<&StatusSlot>) {
        let mut left_slots: Vec<&StatusSlot> = Vec::new();
        let mut center_slots: Vec<&StatusSlot> = Vec::new();
        let mut right_slots: Vec<&StatusSlot> = Vec::new();

        for slot in self.slots.values().filter(|s| s.visible) {
            match slot.alignment {
                SlotAlignment::Left => left_slots.push(slot),
                SlotAlignment::Center => center_slots.push(slot),
                SlotAlignment::Right => right_slots.push(slot),
            }
        }

        // Sort by priority (higher priority first)
        left_slots.sort_by(|a, b| b.priority.cmp(&a.priority));
        center_slots.sort_by(|a, b| b.priority.cmp(&a.priority));
        right_slots.sort_by(|a, b| b.priority.cmp(&a.priority));

        (left_slots, center_slots, right_slots)
    }

    /// Create spans for a group of slots
    fn create_spans_for_slots(&self, slots: &[&StatusSlot]) -> Vec<Span> {
        let mut spans = Vec::new();

        for (i, slot) in slots.iter().enumerate() {
            // Add separator before slot (except for first slot)
            if i > 0 && self.show_separators && !self.separator.is_empty() {
                spans.push(Span::styled(&self.separator, self.background_style));
            }

            // Add the slot content
            let mut content = slot.content.clone();

            // Apply width constraints if specified
            if let Some(max_width) = slot.max_width {
                if content.len() > max_width as usize {
                    content.truncate(max_width as usize - 3);
                    content.push_str("...");
                }
            }

            if let Some(min_width) = slot.min_width {
                if content.len() < min_width as usize {
                    content = format!("{:width$}", content, width = min_width as usize);
                }
            }

            spans.push(Span::styled(content, slot.style));
        }

        spans
    }

    /// Calculate the width needed for a group of spans
    fn calculate_spans_width(&self, spans: &[Span]) -> u16 {
        spans.iter().map(|span| span.content.len() as u16).sum()
    }
}

impl Widget for StatusBar {
    fn render(self, area: Rect, buf: &mut TuiBuffer) {
        // Create the background block
        let block = Block::default()
            .style(self.background_style)
            .borders(Borders::NONE);

        let inner_area = block.inner(area);
        block.render(area, buf);

        if inner_area.width == 0 {
            return;
        }

        // Get organized slots
        let (left_slots, center_slots, right_slots) = self.get_organized_slots();

        // Create spans for each alignment group
        let left_spans = self.create_spans_for_slots(&left_slots);
        let center_spans = self.create_spans_for_slots(&center_slots);
        let right_spans = self.create_spans_for_slots(&right_slots);

        // Calculate widths
        let left_width = self.calculate_spans_width(&left_spans);
        let center_width = self.calculate_spans_width(&center_spans);
        let right_width = self.calculate_spans_width(&right_spans);

        // Calculate layout
        let total_content_width = left_width + center_width + right_width;
        let available_width = inner_area.width;

        if total_content_width <= available_width {
            // We have enough space for all content
            let mut all_spans = Vec::new();

            // Add left-aligned content
            all_spans.extend(left_spans);

            // Calculate center positioning
            let remaining_width = available_width - left_width - right_width;
            if center_width > 0 && remaining_width >= center_width {
                let center_padding = (remaining_width - center_width) / 2;

                // Add padding before center content
                if center_padding > 0 {
                    all_spans.push(Span::styled(
                        " ".repeat(center_padding as usize),
                        self.background_style,
                    ));
                }

                // Add center content
                all_spans.extend(center_spans);

                // Add padding after center content to push right content to the right
                let remaining_padding = remaining_width - center_width - center_padding;
                if remaining_padding > 0 {
                    all_spans.push(Span::styled(
                        " ".repeat(remaining_padding as usize),
                        self.background_style,
                    ));
                }
            } else if center_width == 0 {
                // No center content, pad to push right content to the right
                let padding = available_width - left_width - right_width;
                if padding > 0 {
                    all_spans.push(Span::styled(
                        " ".repeat(padding as usize),
                        self.background_style,
                    ));
                }
            }

            // Add right-aligned content
            all_spans.extend(right_spans);

            let line = Line::from(all_spans);
            let paragraph = Paragraph::new(line).style(self.background_style);
            paragraph.render(inner_area, buf);
        } else {
            // Not enough space, prioritize left content, then right, then center
            let mut truncated_spans = Vec::new();
            let mut used_width = 0u16;

            // Add left content first (highest priority)
            for span in left_spans {
                let span_width = span.content.len() as u16;
                if used_width + span_width <= available_width {
                    used_width += span_width;
                    truncated_spans.push(span);
                } else {
                    break;
                }
            }

            // Add right content next
            let mut right_spans_rev = right_spans;
            right_spans_rev.reverse();
            let mut right_spans_to_add = Vec::new();

            for span in right_spans_rev {
                let span_width = span.content.len() as u16;
                if used_width + span_width <= available_width {
                    used_width += span_width;
                    right_spans_to_add.push(span);
                } else {
                    break;
                }
            }
            right_spans_to_add.reverse();

            // Fill remaining space with padding
            let remaining_width = available_width - used_width;
            if remaining_width > 0 {
                truncated_spans.push(Span::styled(
                    " ".repeat(remaining_width as usize),
                    self.background_style,
                ));
            }

            // Add right spans
            truncated_spans.extend(right_spans_to_add);

            let line = Line::from(truncated_spans);
            let paragraph = Paragraph::new(line).style(self.background_style);
            paragraph.render(inner_area, buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_status_slot_creation() {
        let slot = StatusSlot::new("test", "content")
            .with_alignment(SlotAlignment::Right)
            .with_priority(100)
            .with_style(Style::default().fg(Color::Red));

        assert_eq!(slot.id, "test");
        assert_eq!(slot.content, "content");
        assert_eq!(slot.alignment, SlotAlignment::Right);
        assert_eq!(slot.priority, 100);
    }

    #[test]
    fn test_status_bar_slot_management() {
        let mut status_bar = StatusBar::new();

        let slot = StatusSlot::new("test", "content");
        status_bar.set_slot(slot);

        assert!(status_bar.slots.contains_key("test"));

        status_bar.hide_slot("test");
        assert!(!status_bar.slots.get("test").unwrap().visible);

        status_bar.show_slot("test");
        assert!(status_bar.slots.get("test").unwrap().visible);

        status_bar.update_slot_content("test", "new content");
        assert_eq!(status_bar.slots.get("test").unwrap().content, "new content");

        status_bar.remove_slot("test");
        assert!(!status_bar.slots.contains_key("test"));
    }
}
