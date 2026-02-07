//! Event handling for keyboard input

use std::time::Duration;

use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Application events
#[derive(Debug, Clone, Copy)]
pub enum AppEvent {
    /// Quit the application
    Quit,
    /// Navigate to a specific tab
    NavigateTab(usize),
    /// Move selection up
    Up,
    /// Move selection down
    Down,
    /// Confirm / Enter action
    Enter,
    /// Start selected instance
    Start,
    /// Stop selected instance
    Stop,
    /// Terminate selected instance (with confirmation)
    Terminate,
    /// Refresh data
    Refresh,
    /// Schedule auto-stop
    Schedule,
    /// Show help popup
    ShowHelp,
    /// Open settings dialog
    OpenSettings,
    /// Modify setting value (delta: +1 or -1)
    ModifySettingValue(i32),
    /// Cancel settings dialog
    CancelSettings,
    /// Resize event (width, height)
    Resize(u16, u16),
    /// Start AWS configuration
    ConfigureAws,
    /// Trigger SSO Login
    SsoLogin,
    /// No action
    None,
}

/// Poll for keyboard events with timeout
pub fn poll_event(timeout: Duration) -> Result<Option<AppEvent>> {
    if event::poll(timeout)? {
        match event::read()? {
            Event::Key(key) => {
                if key.kind == KeyEventKind::Press {
                    return Ok(Some(map_key_event(key)));
                }
            }
            Event::Resize(w, h) => {
                return Ok(Some(AppEvent::Resize(w, h)));
            }
            _ => {}
        }
    }
    Ok(None)
}

/// Map key events to application events
fn map_key_event(key: KeyEvent) -> AppEvent {
    match (key.modifiers, key.code) {
        // Quit
        (_, KeyCode::Char('q')) => AppEvent::Quit,
        (KeyModifiers::CONTROL, KeyCode::Char('c')) => AppEvent::Quit,
        
        // Tab navigation
        (_, KeyCode::Char('1')) => AppEvent::NavigateTab(0),
        (_, KeyCode::Char('2')) => AppEvent::NavigateTab(1),
        (_, KeyCode::Char('3')) => AppEvent::NavigateTab(2),
        (_, KeyCode::Char('4')) => AppEvent::NavigateTab(3),
        
        // List navigation
        (_, KeyCode::Up) | (_, KeyCode::Char('k')) => AppEvent::Up,
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => AppEvent::Down,
        (_, KeyCode::Enter) => AppEvent::Enter,
        
        // EC2 actions
        (_, KeyCode::Char('s')) => AppEvent::Start,
        (_, KeyCode::Char('x')) => AppEvent::Stop,
        (_, KeyCode::Char('t')) => AppEvent::Terminate,
        (_, KeyCode::Char('r')) => AppEvent::Refresh,
        (_, KeyCode::Char('a')) => AppEvent::Schedule,
        
        // Help
        (_, KeyCode::Char('?')) => AppEvent::ShowHelp,
        (_, KeyCode::Char('h')) => AppEvent::ShowHelp,
        
        // Settings
        (_, KeyCode::Char(',')) => AppEvent::OpenSettings,
        
        // Tab 5 - Logs
        (_, KeyCode::Char('5')) => AppEvent::NavigateTab(4),
        
        // Settings value modification (Left/Right or -/+)
        (_, KeyCode::Left) | (_, KeyCode::Char('-')) => AppEvent::ModifySettingValue(-1),
        (_, KeyCode::Right) | (_, KeyCode::Char('+')) | (_, KeyCode::Char('=')) => AppEvent::ModifySettingValue(1),
        
        // AWS Config
        (_, KeyCode::Char('c')) => AppEvent::ConfigureAws,
        (_, KeyCode::Char('l')) => AppEvent::SsoLogin,
        
        // Escape - cancel settings or close dialogs
        (_, KeyCode::Esc) => AppEvent::CancelSettings,
        
        _ => AppEvent::None,
    }
}

