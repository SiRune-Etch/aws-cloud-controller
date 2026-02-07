//! Terminal User Interface setup and teardown

use std::io::{stdout, Stdout};

use anyhow::Result;
use crossterm::{
    execute,
    event::{DisableMouseCapture, EnableMouseCapture},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

/// Type alias for our terminal backend
pub type Tui = Terminal<CrosstermBackend<Stdout>>;

/// Initialize the terminal for TUI rendering
pub fn init() -> Result<Tui> {
    execute!(stdout(), EnterAlternateScreen, EnableMouseCapture)?;
    enable_raw_mode()?;
    
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;
    
    // Set up panic hook to restore terminal on crash
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic_info| {
        let _ = restore();
        original_hook(panic_info);
    }));
    
    Ok(terminal)
}

/// Restore terminal to normal state
pub fn restore() -> Result<()> {
    disable_raw_mode()?;
    execute!(stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    Ok(())
}
