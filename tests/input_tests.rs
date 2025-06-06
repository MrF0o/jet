//! Integration tests for input handling and keyboard operations
//!
//! Tests the core input system and keyboard event processing

use editor::events::EventBus;
use editor::input_system::InputSystem;
use ratatui::crossterm::event::{
    KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::time::Duration;

#[tokio::test]
async fn test_input_system_creation() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus);

    // Test that input system can be created
    // The event sender should exist (it's not an Option)
    let _sender = input_system.event_sender();
}

#[tokio::test]
async fn test_key_event_processing() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Create a key event
    let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

    // Process the event (this tests that the input system can handle key events)
    // The actual processing would happen in the handlers, this just tests the pipeline
    let result = input_system.handle_key_input(key_event);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_mouse_event_processing() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Create a mouse event
    let mouse_event = MouseEvent {
        kind: MouseEventKind::Down(MouseButton::Left),
        column: 10,
        row: 5,
        modifiers: KeyModifiers::NONE,
    };

    // Process the event
    let result = input_system.handle_mouse_input(mouse_event);
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_special_key_codes() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test various special key codes
    let special_keys = vec![
        KeyCode::Enter,
        KeyCode::Backspace,
        KeyCode::Delete,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Left,
        KeyCode::Right,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::PageUp,
        KeyCode::PageDown,
        KeyCode::Tab,
        KeyCode::Esc,
    ];

    for key_code in special_keys {
        let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);
        let result = input_system.handle_key_input(key_event);
        assert!(result.is_ok(), "Failed to handle key code: {:?}", key_code);
    }
}

#[tokio::test]
async fn test_key_modifiers() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test various key modifiers
    let modifiers = vec![
        KeyModifiers::NONE,
        KeyModifiers::SHIFT,
        KeyModifiers::CONTROL,
        KeyModifiers::ALT,
        KeyModifiers::SHIFT | KeyModifiers::CONTROL,
        KeyModifiers::CONTROL | KeyModifiers::ALT,
        KeyModifiers::SHIFT | KeyModifiers::ALT,
        KeyModifiers::SHIFT | KeyModifiers::CONTROL | KeyModifiers::ALT,
    ];

    for modifier in modifiers {
        let key_event = KeyEvent::new(KeyCode::Char('a'), modifier);
        let result = input_system.handle_key_input(key_event);
        assert!(result.is_ok(), "Failed to handle modifier: {:?}", modifier);
    }
}

#[tokio::test]
async fn test_character_input() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test various character inputs
    let characters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=[]{}|;':\",./<>?";

    for ch in characters.chars() {
        let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        let result = input_system.handle_key_input(key_event);
        assert!(result.is_ok(), "Failed to handle character: {}", ch);
    }
}

#[tokio::test]
async fn test_mouse_button_types() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test different mouse button types
    let mouse_buttons = vec![MouseButton::Left, MouseButton::Right, MouseButton::Middle];

    for button in mouse_buttons {
        let mouse_event = MouseEvent {
            kind: MouseEventKind::Down(button),
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };

        let result = input_system.handle_mouse_input(mouse_event);
        assert!(
            result.is_ok(),
            "Failed to handle mouse button: {:?}",
            button
        );
    }
}

#[tokio::test]
async fn test_mouse_event_kinds() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test different mouse event kinds
    let event_kinds = vec![
        MouseEventKind::Down(MouseButton::Left),
        MouseEventKind::Up(MouseButton::Left),
        MouseEventKind::Drag(MouseButton::Left),
        MouseEventKind::Moved,
        MouseEventKind::ScrollDown,
        MouseEventKind::ScrollUp,
    ];

    for kind in event_kinds {
        let mouse_event = MouseEvent {
            kind,
            column: 10,
            row: 5,
            modifiers: KeyModifiers::NONE,
        };

        let result = input_system.handle_mouse_input(mouse_event);
        assert!(
            result.is_ok(),
            "Failed to handle mouse event kind: {:?}",
            kind
        );
    }
}

#[tokio::test]
async fn test_input_event_timing() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test that events are processed in a reasonable time
    let key_event = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);

    let start = std::time::Instant::now();
    let result = input_system.handle_key_input(key_event);
    let duration = start.elapsed();

    assert!(result.is_ok(), "Event processing failed");
    assert!(
        duration < Duration::from_millis(50),
        "Event processing took too long: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_rapid_input_processing() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test processing multiple events rapidly
    let events = vec![
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('o'), KeyModifiers::NONE),
    ];

    for event in events {
        let result = input_system.handle_key_input(event);
        assert!(result.is_ok());
    }
}

#[tokio::test]
async fn test_unicode_character_input() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test various Unicode characters
    let unicode_chars = "αβγδε你好世界🚀🎉💻";

    for ch in unicode_chars.chars() {
        let key_event = KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE);
        let result = input_system.handle_key_input(key_event);
        assert!(result.is_ok(), "Failed to handle Unicode character: {}", ch);
    }
}

#[tokio::test]
async fn test_function_keys() {
    let event_bus = EventBus::new();
    let input_system = InputSystem::new(event_bus.clone());

    // Test function keys F1-F12
    for i in 1..=12 {
        let key_code = match i {
            1 => KeyCode::F(1),
            2 => KeyCode::F(2),
            3 => KeyCode::F(3),
            4 => KeyCode::F(4),
            5 => KeyCode::F(5),
            6 => KeyCode::F(6),
            7 => KeyCode::F(7),
            8 => KeyCode::F(8),
            9 => KeyCode::F(9),
            10 => KeyCode::F(10),
            11 => KeyCode::F(11),
            12 => KeyCode::F(12),
            _ => unreachable!(),
        };

        let key_event = KeyEvent::new(key_code, KeyModifiers::NONE);
        let result = input_system.handle_key_input(key_event);
        assert!(result.is_ok(), "Failed to handle function key F{}", i);
    }
}
