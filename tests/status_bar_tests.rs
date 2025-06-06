//! Integration tests for the status bar widget
//!
//! Tests the slot-based status bar system functionality

use editor::widgets::{SlotAlignment, StatusBar, StatusSlot};
use ratatui::{
    backend::TestBackend,
    layout::Rect,
    style::{Color, Style},
    Terminal,
};

#[test]
fn test_status_bar_creation() {
    let status_bar = StatusBar::new();

    // Should start with no slots
    assert_eq!(status_bar.slot_count(), 0);
}

#[test]
fn test_status_slot_creation() {
    let slot = StatusSlot::new("test", "Test Content")
        .with_alignment(SlotAlignment::Right)
        .with_priority(90)
        .with_style(Style::default().fg(Color::Red))
        .with_visibility(false);

    assert_eq!(slot.id, "test");
    assert_eq!(slot.content, "Test Content");
    assert_eq!(slot.alignment, SlotAlignment::Right);
    assert_eq!(slot.priority, 90);
    assert!(!slot.visible);
}

#[test]
fn test_status_bar_slot_management() {
    let mut status_bar = StatusBar::new();

    // Add slots
    let slot1 = StatusSlot::new("file", "main.rs")
        .with_alignment(SlotAlignment::Left)
        .with_priority(100);

    let slot2 = StatusSlot::new("cursor", "Ln 1, Col 1")
        .with_alignment(SlotAlignment::Left)
        .with_priority(90);

    let slot3 = StatusSlot::new("mode", "NORMAL")
        .with_alignment(SlotAlignment::Right)
        .with_priority(100);

    status_bar.set_slot(slot1);
    status_bar.set_slot(slot2);
    status_bar.set_slot(slot3);

    assert_eq!(status_bar.slot_count(), 3);

    // Test slot retrieval
    assert!(status_bar.get_slot("file").is_some());
    assert!(status_bar.get_slot("nonexistent").is_none());

    // Test slot removal
    status_bar.remove_slot("cursor");
    assert_eq!(status_bar.slot_count(), 2);
    assert!(status_bar.get_slot("cursor").is_none());
}

#[test]
fn test_status_bar_slot_visibility() {
    let mut status_bar = StatusBar::new();

    let slot = StatusSlot::new("test", "Test Content").with_visibility(true);

    status_bar.set_slot(slot);

    // Test hiding and showing
    status_bar.hide_slot("test");
    assert!(!status_bar.get_slot("test").unwrap().visible);

    status_bar.show_slot("test");
    assert!(status_bar.get_slot("test").unwrap().visible);
}

#[test]
fn test_status_bar_content_update() {
    let mut status_bar = StatusBar::new();

    let slot = StatusSlot::new("counter", "0");
    status_bar.set_slot(slot);

    // Update content
    status_bar.update_slot_content("counter", "42");
    assert_eq!(status_bar.get_slot("counter").unwrap().content, "42");

    // Test updating non-existent slot (should not panic)
    status_bar.update_slot_content("nonexistent", "value");
}

#[test]
fn test_status_bar_slot_priorities() {
    let mut status_bar = StatusBar::new();

    // Add slots with different priorities (left alignment)
    let high_priority = StatusSlot::new("high", "High")
        .with_alignment(SlotAlignment::Left)
        .with_priority(100);

    let low_priority = StatusSlot::new("low", "Low")
        .with_alignment(SlotAlignment::Left)
        .with_priority(50);

    let medium_priority = StatusSlot::new("medium", "Medium")
        .with_alignment(SlotAlignment::Left)
        .with_priority(75);

    status_bar.set_slot(low_priority);
    status_bar.set_slot(high_priority);
    status_bar.set_slot(medium_priority);

    // Get organized slots and verify ordering
    let (left_slots, _, _) = status_bar.get_organized_slots();

    // Should be ordered by priority (high to low)
    assert_eq!(left_slots[0].id, "high");
    assert_eq!(left_slots[1].id, "medium");
    assert_eq!(left_slots[2].id, "low");
}

#[test]
fn test_status_bar_alignment_groups() {
    let mut status_bar = StatusBar::new();

    // Add slots with different alignments
    let left_slot = StatusSlot::new("left", "Left").with_alignment(SlotAlignment::Left);

    let center_slot = StatusSlot::new("center", "Center").with_alignment(SlotAlignment::Center);

    let right_slot = StatusSlot::new("right", "Right").with_alignment(SlotAlignment::Right);

    status_bar.set_slot(left_slot);
    status_bar.set_slot(center_slot);
    status_bar.set_slot(right_slot);

    let (left_slots, center_slots, right_slots) = status_bar.get_organized_slots();

    assert_eq!(left_slots.len(), 1);
    assert_eq!(center_slots.len(), 1);
    assert_eq!(right_slots.len(), 1);

    assert_eq!(left_slots[0].id, "left");
    assert_eq!(center_slots[0].id, "center");
    assert_eq!(right_slots[0].id, "right");
}

#[test]
fn test_status_bar_rendering() {
    let mut status_bar = StatusBar::new();

    // Add some test slots
    let file_slot = StatusSlot::new("file", "test.rs")
        .with_alignment(SlotAlignment::Left)
        .with_priority(100);

    let mode_slot = StatusSlot::new("mode", "NORMAL")
        .with_alignment(SlotAlignment::Right)
        .with_priority(100);

    status_bar.set_slot(file_slot);
    status_bar.set_slot(mode_slot);

    // Create a test backend and render
    let backend = TestBackend::new(80, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal
        .draw(|f| {
            let area = Rect::new(0, 0, 80, 1);
            f.render_widget(status_bar.clone(), area);
        })
        .unwrap();

    // The test passes if rendering doesn't panic
    // More detailed rendering tests would require inspecting the buffer content
}

#[test]
fn test_status_bar_hidden_slots() {
    let mut status_bar = StatusBar::new();

    // Add visible and hidden slots
    let visible_slot = StatusSlot::new("visible", "Visible")
        .with_alignment(SlotAlignment::Left)
        .with_visibility(true);

    let hidden_slot = StatusSlot::new("hidden", "Hidden")
        .with_alignment(SlotAlignment::Left)
        .with_visibility(false);

    status_bar.set_slot(visible_slot);
    status_bar.set_slot(hidden_slot);

    let (left_slots, _, _) = status_bar.get_organized_slots();

    // Only visible slots should be returned
    assert_eq!(left_slots.len(), 1);
    assert_eq!(left_slots[0].id, "visible");
}

#[test]
fn test_status_bar_width_constraints() {
    let slot = StatusSlot::new("test", "Test Content")
        .with_min_width(10)
        .with_max_width(20);

    assert_eq!(slot.min_width, Some(10));
    assert_eq!(slot.max_width, Some(20));
}

#[test]
fn test_status_bar_style_customization() {
    let custom_style = Style::default().fg(Color::Yellow).bg(Color::Blue);

    let slot = StatusSlot::new("styled", "Styled Content").with_style(custom_style);

    assert_eq!(slot.style, custom_style);
}

#[test]
fn test_status_bar_clone() {
    let mut status_bar = StatusBar::new();

    let slot = StatusSlot::new("test", "Test Content");
    status_bar.set_slot(slot);

    // Test that StatusBar can be cloned
    let cloned_bar = status_bar.clone();
    assert_eq!(cloned_bar.slot_count(), 1);
    assert!(cloned_bar.get_slot("test").is_some());
}
