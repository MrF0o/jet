//! Integration tests for the application state and core functionality
//!
//! Tests the main App struct and its key operations

use std::fs;
use tempfile::TempDir;

use editor::{buffer::Buffer, App};

#[tokio::test]
async fn test_app_creation() {
    let app = App::new().await;

    assert!(app.running);
    assert_eq!(app.buffers.len(), 1);
    assert_eq!(app.active_buffer, 0);
    assert_eq!(app.scroll_offset, (0, 0));
    assert!(matches!(app.command_mode, editor::app::CommandMode::Normal));
    assert_eq!(app.command_input, "");
    assert!(app.status_message.is_none());
    assert!(!app.show_command_palette);
    assert!(app.mouse_drag_start.is_none());

    // Test that status bar is initialized with default slots
    assert!(app.status_bar.slot_count() > 0);
}

#[tokio::test]
async fn test_app_with_file() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a test file
    fs::write(&file_path, "Hello World\nSecond Line").unwrap();

    // Create app with file
    let app = App::with_file(file_path.to_str().unwrap()).await.unwrap();

    assert!(app.running);
    assert_eq!(app.buffers.len(), 1);
    assert_eq!(app.buffers[0].name, "test.txt");
    assert_eq!(app.buffers[0].content.len(), 2);
    assert_eq!(app.buffers[0].content[0], "Hello World");
    assert_eq!(app.buffers[0].content[1], "Second Line");
    assert!(!app.buffers[0].modified);

    // Test that status bar is initialized
    assert!(app.status_bar.slot_count() > 0);
}

#[tokio::test]
async fn test_app_buffer_management() {
    let mut app = App::new().await;

    // Initially should have one empty buffer
    assert_eq!(app.buffers.len(), 1);
    assert_eq!(app.active_buffer, 0);

    // Test buffer operations through app
    let buffer = &mut app.buffers[app.active_buffer];
    buffer.insert_char('H');
    buffer.insert_char('i');

    assert_eq!(buffer.content[0], "Hi");
    assert!(buffer.modified);
}

#[tokio::test]
async fn test_app_status_bar_updates() {
    let mut app = App::new().await;

    // Modify buffer content
    let buffer = &mut app.buffers[app.active_buffer];
    buffer.insert_char('T');
    buffer.insert_char('e');
    buffer.insert_char('s');
    buffer.insert_char('t');
    buffer.cursor_pos = (0, 2); // Position cursor in middle

    // Update status bar
    app.update_status_bar();

    // Check that status bar slots are updated with current state
    let file_slot = app.status_bar.get_slot("file_info");
    assert!(file_slot.is_some());

    let cursor_slot = app.status_bar.get_slot("cursor_pos");
    assert!(cursor_slot.is_some());
    assert!(cursor_slot.unwrap().content.contains("Ln 1, Col 3")); // 1-based indexing

    let modified_slot = app.status_bar.get_slot("modified_status");
    assert!(modified_slot.is_some());
    assert!(modified_slot.unwrap().content.contains("Unsaved"));

    let mode_slot = app.status_bar.get_slot("mode_indicator");
    assert!(mode_slot.is_some());
    assert!(mode_slot.unwrap().content.contains("NORMAL"));
}

#[tokio::test]
async fn test_app_status_bar_selection_info() {
    let mut app = App::new().await;

    // Add some content
    let buffer = &mut app.buffers[app.active_buffer];
    for ch in "Hello World".chars() {
        buffer.insert_char(ch);
    }

    // Create a selection
    buffer.cursor_pos = (0, 0);
    buffer.toggle_visual_mode();
    buffer.cursor_pos = (0, 5); // Select "Hello"

    // Update status bar
    app.update_status_bar();

    // Check that selection info slot is visible and has content
    let selection_slot = app.status_bar.get_slot("selection_info");
    assert!(selection_slot.is_some());
    assert!(selection_slot.unwrap().visible);
    assert!(selection_slot.unwrap().content.contains("5 chars"));
}

#[tokio::test]
async fn test_app_status_bar_no_selection() {
    let mut app = App::new().await;

    // Add content but no selection
    let buffer = &mut app.buffers[app.active_buffer];
    buffer.insert_char('T');
    buffer.insert_char('e');
    buffer.insert_char('s');
    buffer.insert_char('t');

    // Update status bar
    app.update_status_bar();

    // Check that selection info slot is hidden when no selection
    let selection_slot = app.status_bar.get_slot("selection_info");
    assert!(selection_slot.is_some());
    assert!(!selection_slot.unwrap().visible);
}

#[tokio::test]
async fn test_app_command_mode_switching() {
    let mut app = App::new().await;

    // Test initial state
    assert!(matches!(app.command_mode, editor::app::CommandMode::Normal));

    // Test command mode switching
    app.command_mode = editor::app::CommandMode::Command;
    assert!(matches!(
        app.command_mode,
        editor::app::CommandMode::Command
    ));

    app.command_mode = editor::app::CommandMode::FileSearch;
    assert!(matches!(
        app.command_mode,
        editor::app::CommandMode::FileSearch
    ));

    app.command_mode = editor::app::CommandMode::TextSearch;
    assert!(matches!(
        app.command_mode,
        editor::app::CommandMode::TextSearch
    ));
}

#[tokio::test]
async fn test_app_command_palette_state() {
    let mut app = App::new().await;

    // Test initial state
    assert!(!app.show_command_palette);
    assert_eq!(app.command_input, "");

    // Test showing command palette
    app.show_command_palette = true;
    app.command_input = "test command".to_string();

    assert!(app.show_command_palette);
    assert_eq!(app.command_input, "test command");
}

#[tokio::test]
async fn test_app_scroll_offset() {
    let mut app = App::new().await;

    // Test initial scroll offset
    assert_eq!(app.scroll_offset, (0, 0));

    // Test setting scroll offset
    app.scroll_offset = (10, 5);
    assert_eq!(app.scroll_offset, (10, 5));
}

#[tokio::test]
async fn test_app_status_message() {
    let mut app = App::new().await;

    // Test initial state
    assert!(app.status_message.is_none());

    // Test setting status message
    app.status_message = Some("Test message".to_string());
    assert_eq!(app.status_message, Some("Test message".to_string()));
}

#[tokio::test]
async fn test_app_clone() {
    let app = App::new().await;

    // Test that App can be cloned
    let cloned_app = app.clone();

    assert_eq!(cloned_app.running, app.running);
    assert_eq!(cloned_app.buffers.len(), app.buffers.len());
    assert_eq!(cloned_app.active_buffer, app.active_buffer);
    assert_eq!(cloned_app.scroll_offset, app.scroll_offset);
    assert!(matches!(
        cloned_app.command_mode,
        editor::app::CommandMode::Normal
    ));

    // Status bar should be reinitialized
    assert!(cloned_app.status_bar.slot_count() > 0);
}

#[tokio::test]
async fn test_app_default() {
    let app = App::default();

    assert!(app.running);
    assert_eq!(app.buffers.len(), 1);
    assert_eq!(app.active_buffer, 0);
    assert!(matches!(app.command_mode, editor::app::CommandMode::Normal));
    assert_eq!(app.command_input, "");
    assert!(!app.show_command_palette);

    // Status bar should be initialized
    assert!(app.status_bar.slot_count() > 0);
}

#[tokio::test]
async fn test_app_multiple_buffers() {
    let mut app = App::new().await;

    // Add more buffers
    app.buffers.push(Buffer::new());
    app.buffers.push(Buffer::new());

    assert_eq!(app.buffers.len(), 3);

    // Test switching between buffers
    app.active_buffer = 1;
    assert_eq!(app.active_buffer, 1);

    app.active_buffer = 2;
    assert_eq!(app.active_buffer, 2);

    // Update status bar and check buffer count
    app.update_status_bar();
    let buffer_count_slot = app.status_bar.get_slot("buffer_count");
    assert!(buffer_count_slot.is_some());
    assert!(buffer_count_slot.unwrap().content.contains("3"));
}
