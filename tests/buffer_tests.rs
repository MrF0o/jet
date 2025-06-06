//! Integration tests for buffer functionality
//!
//! Tests the core text buffer operations that are fundamental to the editor

use std::fs;
use tempfile::TempDir;

use editor::buffer::Buffer;

#[tokio::test]
async fn test_buffer_creation() {
    let buffer = Buffer::new();

    assert_eq!(buffer.name, "untitled");
    assert_eq!(buffer.content.len(), 1); // Should have one empty line
    assert_eq!(buffer.cursor_pos, (0, 0));
    assert!(!buffer.modified);
    assert_eq!(buffer.selection_start, None);
}

#[tokio::test]
async fn test_buffer_text_insertion() {
    let mut buffer = Buffer::new();

    // Insert some text
    buffer.insert_char('H');
    buffer.insert_char('e');
    buffer.insert_char('l');
    buffer.insert_char('l');
    buffer.insert_char('o');

    assert_eq!(buffer.content[0], "Hello");
    assert_eq!(buffer.cursor_pos, (0, 5));
    assert!(buffer.modified);
}

#[tokio::test]
async fn test_buffer_newline_insertion() {
    let mut buffer = Buffer::new();

    buffer.insert_char('H');
    buffer.insert_char('i');
    buffer.insert_newline();
    buffer.insert_char('B');
    buffer.insert_char('y');
    buffer.insert_char('e');

    assert_eq!(buffer.content.len(), 2);
    assert_eq!(buffer.content[0], "Hi");
    assert_eq!(buffer.content[1], "Bye");
    assert_eq!(buffer.cursor_pos, (1, 3));
}

#[tokio::test]
async fn test_buffer_backspace() {
    let mut buffer = Buffer::new();

    // Insert text then backspace
    buffer.insert_char('H');
    buffer.insert_char('e');
    buffer.insert_char('l');
    buffer.insert_char('l');
    buffer.insert_char('o');

    buffer.backspace();
    buffer.backspace();

    assert_eq!(buffer.content[0], "Hel");
    assert_eq!(buffer.cursor_pos, (0, 3));
}

#[tokio::test]
async fn test_buffer_cursor_movement() {
    let mut buffer = Buffer::new();

    // Create some content
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

    // Test cursor movements
    buffer.move_cursor(editor::buffer::CursorMovement::Left);
    assert_eq!(buffer.cursor_pos, (1, 4));

    buffer.move_cursor(editor::buffer::CursorMovement::Up);
    assert_eq!(buffer.cursor_pos, (0, 4));

    buffer.move_cursor(editor::buffer::CursorMovement::Right);
    assert_eq!(buffer.cursor_pos, (0, 5));

    buffer.move_cursor(editor::buffer::CursorMovement::Down);
    assert_eq!(buffer.cursor_pos, (1, 5));
}

#[tokio::test]
async fn test_buffer_text_selection() {
    let mut buffer = Buffer::new();

    // Insert some text
    for ch in "Hello World".chars() {
        buffer.insert_char(ch);
    }

    // Move cursor and start selection
    buffer.cursor_pos = (0, 0);
    buffer.toggle_visual_mode();

    // Move cursor to select "Hello"
    buffer.cursor_pos = (0, 5);

    let selected_text = buffer.get_selected_text();
    assert_eq!(selected_text, Some("Hello".to_string()));
}

#[tokio::test]
async fn test_buffer_file_operations() {
    let temp_dir = TempDir::new().unwrap();
    let file_path = temp_dir.path().join("test.txt");

    // Create a test file
    fs::write(&file_path, "Hello\nWorld\nTest").unwrap();

    // Load buffer from file
    let buffer = Buffer::from_path_async(file_path.clone()).await.unwrap();

    assert_eq!(buffer.name, "test.txt");
    assert_eq!(buffer.content.len(), 3);
    assert_eq!(buffer.content[0], "Hello");
    assert_eq!(buffer.content[1], "World");
    assert_eq!(buffer.content[2], "Test");
    assert!(!buffer.modified);

    // Test saving
    let mut buffer = buffer;
    buffer.insert_char('!');
    assert!(buffer.modified);

    buffer.save_to_path_async(file_path.clone()).await.unwrap();
    assert!(!buffer.modified);

    // Verify file was saved correctly
    let content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(content, "!Hello\nWorld\nTest");
}

#[tokio::test]
async fn test_buffer_multiline_selection() {
    let mut buffer = Buffer::new();

    // Create multiline content
    for ch in "Line 1\nLine 2\nLine 3".chars() {
        if ch == '\n' {
            buffer.insert_newline();
        } else {
            buffer.insert_char(ch);
        }
    }

    // Select from start of line 1 to middle of line 2
    buffer.cursor_pos = (0, 0);
    buffer.toggle_visual_mode();
    buffer.cursor_pos = (1, 3);

    let selected_text = buffer.get_selected_text();
    assert_eq!(selected_text, Some("Line 1\nLin".to_string()));
}

#[tokio::test]
async fn test_buffer_line_boundaries() {
    let mut buffer = Buffer::new();

    // Test cursor movement at line boundaries
    buffer.move_cursor(editor::buffer::CursorMovement::Left); // Should not go below (0, 0)
    assert_eq!(buffer.cursor_pos, (0, 0));

    buffer.move_cursor(editor::buffer::CursorMovement::Up); // Should not go above (0, 0)
    assert_eq!(buffer.cursor_pos, (0, 0));

    // Add some content and test right boundary
    buffer.insert_char('H');
    buffer.insert_char('i');
    buffer.cursor_pos = (0, 0);

    buffer.move_cursor(editor::buffer::CursorMovement::Right);
    buffer.move_cursor(editor::buffer::CursorMovement::Right);
    buffer.move_cursor(editor::buffer::CursorMovement::Right); // Should not go beyond line end
    assert_eq!(buffer.cursor_pos, (0, 2));
}
