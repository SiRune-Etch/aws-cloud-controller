//! AWS Cloud Controller - TUI for managing AWS resources
//!
//! A terminal-based interface for EC2 instance management and Lambda function control.

mod app;
mod aws;
mod config;
mod event;
mod logger;
mod settings;
mod tui;
mod ui;

use anyhow::Result;
use app::App;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging to file (avoid terminal output)
    let file_appender = tracing_subscriber::fmt::layer()
        .with_ansi(false)
        .with_writer(std::io::sink);
    
    tracing_subscriber::registry()
        .with(file_appender)
        .init();

    // Initialize and run the application
    let mut app = App::new().await?;
    let mut terminal = tui::init()?;
    
    // Set initial window size
    if let Ok(size) = terminal.size() {
        app.window_size = (size.width, size.height);
    }

    let result = app.run(&mut terminal).await;

    // Restore terminal state before handling any errors
    tui::restore()?;

    result
}
