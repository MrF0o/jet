use anyhow::Result;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    crossterm::{
        event::{DisableMouseCapture, EnableMouseCapture},
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    },
};
use std::io::stdout;

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

// Re-export main types for easier imports
pub use app::{App, CommandMode};

#[tokio::main]
async fn main() -> Result<()> {
    // Get command line arguments
    let args: Vec<String> = std::env::args().collect();

    // Setup terminal - disable mouse events to prevent OS text selection
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

    // Create backend without mouse events
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and run the app
    let mut app = if args.len() > 1 {
        App::with_file(&args[1]).await?
    } else {
        App::new().await
    };
    let result = app.run(&mut terminal).await;

    // Restore the terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        ratatui::crossterm::cursor::Show
    )?;

    // Handle any final errors
    match result {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("Error: {}", e);
            Err(e)
        }
    }
}
