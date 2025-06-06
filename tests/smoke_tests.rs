//! Smoke tests for overall application functionality
//!
//! These are high-level tests that verify the application works end-to-end

use editor::App;
use std::fs;
use tempfile::TempDir;

#[tokio::test]
async fn test_app_smoke_test() {
    // Test that we can create and initialize an app
    let app = App::new().await;

    assert!(app.running);
    assert!(!app.buffers.is_empty());
    assert!(app.status_bar.slot_count() > 0);
}

#[tokio::test]
async fn test_file_loading_smoke_test() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("smoke_test.txt");

    // Create a test file
    fs::write(&file_path, "Line 1\nLine 2\nLine 3").unwrap();

    // Test loading file
    let app = App::with_file(file_path.to_str().unwrap()).await.unwrap();

    assert_eq!(app.buffers[0].content.len(), 3);
    assert_eq!(app.buffers[0].content[0], "Line 1");
    assert_eq!(app.buffers[0].name, "smoke_test.txt");
}

#[tokio::test]
async fn test_basic_editing_smoke_test() {
    let mut app = App::new().await;

    // Test basic editing operations
    let buffer = &mut app.buffers[app.active_buffer];

    // Insert text
    for ch in "Hello World".chars() {
        buffer.insert_char(ch);
    }

    assert_eq!(buffer.content[0], "Hello World");
    assert!(buffer.modified);

    // Test cursor movement
    buffer.cursor_pos = (0, 0);
    buffer.move_cursor(editor::buffer::CursorMovement::Right);
    buffer.move_cursor(editor::buffer::CursorMovement::Right);
    assert_eq!(buffer.cursor_pos, (0, 2));

    // Test backspace
    buffer.backspace();
    assert_eq!(buffer.content[0], "Hllo World");
    assert_eq!(buffer.cursor_pos, (0, 1));
}

#[tokio::test]
async fn test_status_bar_integration_smoke_test() {
    let mut app = App::new().await;

    // Modify buffer
    let buffer = &mut app.buffers[app.active_buffer];
    buffer.insert_char('T');
    buffer.insert_char('e');
    buffer.insert_char('s');
    buffer.insert_char('t');

    // Update status bar
    app.update_status_bar();

    // Verify status bar has expected slots
    assert!(app.status_bar.get_slot("file_info").is_some());
    assert!(app.status_bar.get_slot("cursor_pos").is_some());
    assert!(app.status_bar.get_slot("modified_status").is_some());
    assert!(app.status_bar.get_slot("mode_indicator").is_some());

    // Verify content is updated
    let cursor_slot = app.status_bar.get_slot("cursor_pos").unwrap();
    assert!(cursor_slot.content.contains("Ln"));
    assert!(cursor_slot.content.contains("Col"));

    let modified_slot = app.status_bar.get_slot("modified_status").unwrap();
    assert!(modified_slot.content.contains("Unsaved"));
}

#[tokio::test]
async fn test_selection_smoke_test() {
    let mut app = App::new().await;

    // Add content
    let buffer = &mut app.buffers[app.active_buffer];
    for ch in "Hello World Test".chars() {
        buffer.insert_char(ch);
    }

    // Create selection
    buffer.cursor_pos = (0, 6); // After "Hello "
    buffer.toggle_visual_mode();
    buffer.cursor_pos = (0, 11); // Select "World"

    let selected_text = buffer.get_selected_text();
    assert_eq!(selected_text, Some("World".to_string()));

    // Update status bar and check selection info
    app.update_status_bar();
    let selection_slot = app.status_bar.get_slot("selection_info").unwrap();
    assert!(selection_slot.visible);
    assert!(selection_slot.content.contains("5 chars"));
}

#[tokio::test]
async fn test_multiline_editing_smoke_test() {
    let mut app = App::new().await;

    let buffer = &mut app.buffers[app.active_buffer];

    // Create multiline content
    buffer.insert_char('L');
    buffer.insert_char('i');
    buffer.insert_char('n');
    buffer.insert_char('e');
    buffer.insert_char(' ');
    buffer.insert_char('1');
    buffer.insert_newline();
    buffer.insert_char('L');
    buffer.insert_char('i');
    buffer.insert_char('n');
    buffer.insert_char('e');
    buffer.insert_char(' ');
    buffer.insert_char('2');

    assert_eq!(buffer.content.len(), 2);
    assert_eq!(buffer.content[0], "Line 1");
    assert_eq!(buffer.content[1], "Line 2");
    assert_eq!(buffer.cursor_pos, (1, 6));

    // Test multiline navigation
    buffer.move_cursor(editor::buffer::CursorMovement::Up);
    assert_eq!(buffer.cursor_pos, (0, 6));

    buffer.move_cursor(editor::buffer::CursorMovement::Down);
    assert_eq!(buffer.cursor_pos, (1, 6));
}

#[tokio::test]
async fn test_app_state_consistency() {
    let mut app = App::new().await;

    // Perform various operations and check state consistency
    let buffer = &mut app.buffers[app.active_buffer];

    // Initial state
    assert_eq!(buffer.cursor_pos, (0, 0));
    assert!(!buffer.modified);

    // Insert text
    buffer.insert_char('A');
    assert!(buffer.modified);
    assert_eq!(buffer.cursor_pos, (0, 1));

    // Add newline
    buffer.insert_newline();
    assert_eq!(buffer.cursor_pos, (1, 0));
    assert_eq!(buffer.content.len(), 2);

    // Backspace across lines
    buffer.backspace();
    assert_eq!(buffer.cursor_pos, (0, 1));
    assert_eq!(buffer.content.len(), 1);

    // State should remain consistent
    assert!(buffer.modified);
}

#[tokio::test]
async fn test_command_mode_smoke_test() {
    let mut app = App::new().await;

    // Test command mode state
    assert!(matches!(app.command_mode, editor::app::CommandMode::Normal));
    assert!(!app.show_command_palette);
    assert_eq!(app.command_input, "");

    // Simulate entering command mode
    app.command_mode = editor::app::CommandMode::Command;
    app.show_command_palette = true;
    app.command_input = "test".to_string();

    assert!(matches!(
        app.command_mode,
        editor::app::CommandMode::Command
    ));
    assert!(app.show_command_palette);
    assert_eq!(app.command_input, "test");
}
