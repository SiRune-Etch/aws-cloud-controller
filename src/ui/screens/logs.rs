use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::logger::LogLevel;
use crate::ui::utils::pad_rect;

/// Render logs screen
pub fn render_logs(frame: &mut Frame, app: &App, area: Rect) {
    // Render outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" Activity Logs ({}) ", app.log_manager.entries().len()))
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(outer_block.clone(), area);
    
    // Get padded inner area
    let inner_area = outer_block.inner(area);
    let padded_area = pad_rect(inner_area, 1, 1, 0, 0);
    
    if app.log_manager.entries().is_empty() {
        let msg = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled("No log entries yet.", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("Logs will appear here as you perform actions.", Style::default().fg(Color::DarkGray))),
        ])
        .block(Block::default());
        frame.render_widget(msg, padded_area);
        return;
    }
    
    // Create log lines (showing most recent at the bottom)
    let visible_height = padded_area.height as usize;
    let entries = app.log_manager.entries();
    
    // Filter entries based on verbosity setting
    let filtered_entries: Vec<&crate::logger::LogEntry> = entries.iter()
        .filter(|e| app.settings.should_show_log(e.level))
        .collect();
        
    let scroll_offset = app.log_manager.scroll_offset();
    
    // Calculate which entries to show
    let start_idx = filtered_entries.len().saturating_sub(visible_height + scroll_offset);
    let end_idx = filtered_entries.len().saturating_sub(scroll_offset);
    
    let log_lines: Vec<Line> = filtered_entries[start_idx..end_idx]
        .iter()
        .map(|entry| {
            let (level_style, level_icon) = match entry.level {
                LogLevel::Debug => (Style::default().fg(Color::Magenta), "🔍"),
                LogLevel::Info => (Style::default().fg(Color::Cyan), "ℹ"),
                LogLevel::Success => (Style::default().fg(Color::Green), "✓"),
                LogLevel::Warning => (Style::default().fg(Color::Yellow), "⚠"),
                LogLevel::Error => (Style::default().fg(Color::Red), "✗"),
            };
            
            let timestamp = entry.timestamp.format("%H:%M:%S").to_string();
            
            // Truncate long messages to prevent UI clutter
            let display_message = if entry.message.len() > 150 {
                format!("{}...", &entry.message[0..150])
            } else {
                entry.message.clone()
            };
            
            Line::from(vec![
                Span::styled(format!(" {} ", level_icon), level_style),
                Span::styled(format!("[{}] ", timestamp), Style::default().fg(Color::DarkGray)),
                Span::styled(display_message, level_style),
            ])
        })
        .collect();
    
    let logs = Paragraph::new(log_lines)
        .block(Block::default())
        .wrap(Wrap { trim: false });
    
    frame.render_widget(logs, padded_area);
}
