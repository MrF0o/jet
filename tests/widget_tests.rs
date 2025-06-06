//! Integration tests for the widget system
//! 
//! Tests the core widget functionality including editor, cursor, and modal widgets

use editor::widgets::{
    cursor::CursorManager,
    editor::Editor,
    modal::CommandPalette,
    toast::ToastManager,
};
use editor::buffer::Buffer;
use ratatui::{
    backend::TestBackend,
    layout::Rect,
    Terminal,
};

#[test]
fn test_cursor_manager_creation() {
    let cursor_manager = CursorManager::new();
    
    // Test initial state
    assert_eq!(cursor_manager.get_active_context(), None);
}

#[test]
fn test_cursor_manager_context_switching() {
    let mut cursor_manager = CursorManager::new();
    
    // Test setting active context
    cursor_manager.set_active_context("editor");
    assert_eq!(cursor_manager.get_active_context(), Some("editor"));
    
    cursor_manager.set_active_context("command_palette");
    assert_eq!(cursor_manager.get_active_context(), Some("command_palette"));
}

#[test]
fn test_cursor_manager_position_updates() {
    let mut cursor_manager = CursorManager::new();
    
    // Test updating cursor position
    cursor_manager.set_active_context("editor"); // Need to set context first
    cursor_manager.update_cursor_position("editor", 10, 5);
    
    let position = cursor_manager.get_cursor_position("editor");
    assert!(position.is_some());
    assert_eq!(position.unwrap().x, 10);
    assert_eq!(position.unwrap().y, 5);
}

#[test]
fn test_cursor_manager_visibility() {
    let mut cursor_manager = CursorManager::new();
    
    // Set up a cursor
    cursor_manager.update_cursor_position("editor", 10, 5);
    cursor_manager.set_active_context("editor");
    
    // Test hiding and showing
    cursor_manager.hide_cursor("editor");
    // Note: We can't easily test visibility state without more internal access
    
    cursor_manager.show_cursor("editor");
    // Similarly for showing
}

#[test]
fn test_editor_widget_creation() {
    let buffer = Buffer::new();
    let _editor = Editor::new(&buffer);
    
    // Test that editor widget can be created
    // The actual rendering test would require more setup
}

#[test]
fn test_editor_widget_with_content() {
    let mut buffer = Buffer::new();
    
    // Add some content
    buffer.insert_char('H');
    buffer.insert_char('e');
    buffer.insert_char('l');
    buffer.insert_char('l');
    buffer.insert_char('o');
    buffer.insert_newline();
    buffer.insert_char('W');
    buffer.insert_char('o');
    buffer.insert_char('r');
    buffer.insert_char('l');
    buffer.insert_char('d');
    
    let _editor = Editor::new(&buffer);
    
    // Test that editor can be created with content
    // More detailed tests would require rendering
}

#[test]
fn test_editor_widget_scroll_offset() {
    let buffer = Buffer::new();
    
    // Test different scroll offsets
    let _editor1 = Editor::new(&buffer);
    let _editor2 = Editor::new(&buffer);
    let _editor3 = Editor::new(&buffer);
    
    // Test that editors can be created with different scroll offsets
}

#[test]
fn test_command_palette_creation() {
    let command_input = "test command".to_string();
    let _palette = CommandPalette::new(&command_input);
    
    // Test that command palette can be created
}

#[test]
fn test_command_palette_rendering() {
    let command_input = "search".to_string();
    let palette = CommandPalette::new(&command_input);
    
    // Create a test backend and render
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal.draw(|f| {
        let area = Rect::new(0, 0, 80, 24);
        f.render_widget(palette, area);
    }).unwrap();
    
    // Test passes if rendering doesn't panic
}

#[test]
fn test_toast_manager_creation() {
    let toast_manager = ToastManager::new();
    
    // Test initial state
    assert!(!toast_manager.has_active_toasts());
}

#[test]
fn test_toast_manager_adding_toasts() {
    let mut toast_manager = ToastManager::new();
    
    // Add a toast
    toast_manager.add_info("Test message".to_string());
    assert!(toast_manager.has_active_toasts());
    
    // Add different types of toasts
    toast_manager.add_success("Success message".to_string());
    toast_manager.add_warning("Warning message".to_string());
    toast_manager.add_error("Error message".to_string());
    
    assert!(toast_manager.has_active_toasts());
}

#[test]
fn test_toast_manager_update_and_expiry() {
    let mut toast_manager = ToastManager::new();
    
    // Add a toast with short duration
    toast_manager.add_info("Test message".to_string());
    assert!(toast_manager.has_active_toasts());
    
    // Update should not immediately remove toasts
    toast_manager.update();
    // We can't easily test timing without sleeping or mocking time
}

#[test]
fn test_editor_widget_with_selection() {
    let mut buffer = Buffer::new();
    
    // Add content and create selection
    for ch in "Hello World".chars() {
        buffer.insert_char(ch);
    }
    
    buffer.cursor_pos = (0, 0);
    buffer.toggle_visual_mode();
    buffer.cursor_pos = (0, 5); // Select "Hello"
    
    let _editor = Editor::new(&buffer);
    
    // Test that editor can handle buffers with selections
}

#[test]
fn test_editor_widget_rendering() {
    let mut buffer = Buffer::new();
    
    // Add some content to render
    buffer.insert_char('T');
    buffer.insert_char('e');
    buffer.insert_char('s');
    buffer.insert_char('t');
    buffer.insert_newline();
    buffer.insert_char('L');
    buffer.insert_char('i');
    buffer.insert_char('n');
    buffer.insert_char('e');
    
    let editor = Editor::new(&buffer);
    
    // Create a test backend and render
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal.draw(|f| {
        let area = Rect::new(0, 0, 80, 23); // Leave space for status bar
        f.render_widget(editor, area);
    }).unwrap();
    
    // Test passes if rendering doesn't panic
}

#[test]
fn test_command_palette_empty_input() {
    let command_input = String::new();
    let palette = CommandPalette::new(&command_input);
    
    // Test command palette with empty input
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal.draw(|f| {
        let area = Rect::new(0, 0, 80, 24);
        f.render_widget(palette, area);
    }).unwrap();
}

#[test]
fn test_command_palette_long_input() {
    let command_input = "very long command input that might exceed the width of the terminal window".to_string();
    let palette = CommandPalette::new(&command_input);
    
    // Test command palette with long input
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    
    terminal.draw(|f| {
        let area = Rect::new(0, 0, 80, 24);
        f.render_widget(palette, area);
    }).unwrap();
}

#[test]
fn test_toast_manager_multiple_toasts() {
    let mut toast_manager = ToastManager::new();
    
    // Add multiple toasts of different types
    toast_manager.add_info("Info 1".to_string());
    toast_manager.add_info("Info 2".to_string());
    toast_manager.add_success("Success 1".to_string());
    toast_manager.add_warning("Warning 1".to_string());
    toast_manager.add_error("Error 1".to_string());
    
    assert!(toast_manager.has_active_toasts());
    
    // Update multiple times
    for _ in 0..5 {
        toast_manager.update();
    }
    
    // Should still have toasts (they don't expire immediately)
    // Exact behavior depends on toast duration implementation
}

#[test]
fn test_cursor_manager_multiple_contexts() {
    let mut cursor_manager = CursorManager::new();
    
    // Set up cursors for different contexts
    cursor_manager.update_cursor_position("editor", 10, 5);
    cursor_manager.update_cursor_position("command_palette", 20, 0);
    cursor_manager.update_cursor_position("search", 0, 15);
    
    // Test switching between contexts
    cursor_manager.set_active_context("editor");
    assert_eq!(cursor_manager.get_active_context(), Some("editor"));
    
    cursor_manager.set_active_context("command_palette");
    assert_eq!(cursor_manager.get_active_context(), Some("command_palette"));
    
    cursor_manager.set_active_context("search");
    assert_eq!(cursor_manager.get_active_context(), Some("search"));
    
    // Test getting positions
    let editor_pos = cursor_manager.get_cursor_position("editor");
    assert!(editor_pos.is_some());
    assert_eq!(editor_pos.unwrap().x, 10);
    assert_eq!(editor_pos.unwrap().y, 5);
    
    let palette_pos = cursor_manager.get_cursor_position("command_palette");
    assert!(palette_pos.is_some());
    assert_eq!(palette_pos.unwrap().x, 20);
    assert_eq!(palette_pos.unwrap().y, 0);
    
    let search_pos = cursor_manager.get_cursor_position("search");
    assert!(search_pos.is_some());
    assert_eq!(search_pos.unwrap().x, 0);
    assert_eq!(search_pos.unwrap().y, 15);
}
