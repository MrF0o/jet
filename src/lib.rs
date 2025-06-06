//! Editor library for testing purposes

pub mod app;
pub mod buffer;
pub mod config;
pub mod events;
pub mod handlers;
pub mod input;
pub mod input_system;
pub mod performance;
pub mod plugins;
pub mod ui;
pub mod widgets;

// Re-export main types for convenience
pub use app::{App, CommandMode};
pub use buffer::Buffer;
