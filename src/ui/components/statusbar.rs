use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::App;

/// Render status bar with control hints
pub fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(52)]) // Increased from 35 to fit controls
        .split(area);

    // Left side: Status message with refresh timer
    let loading_indicator = if app.is_loading { "⏳ " } else { "" };
    let alert_count = if app.pending_alerts.is_empty() {
        String::new()
    } else {
        format!(" | 🔔 {} alerts", app.pending_alerts.len())
    };
    
    // Build refresh timer text
    let refresh_text = if app.is_loading {
        " | Refreshing...".to_string()
    } else if let Some(seconds) = app.seconds_until_refresh() {
        format!(" | Next refresh: {}s", seconds)
    } else {
        String::new()
    };

    let status_display = if app.status_message.len() > 100 {
        format!("{}...", &app.status_message[0..100])
    } else {
        app.status_message.clone()
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(loading_indicator, Style::default().fg(Color::Yellow)),
        Span::styled(status_display, Style::default().fg(Color::White)),
        Span::styled(
            format!(" | Region: {}", app.aws_client.region),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(alert_count, Style::default().fg(Color::Red)),
        Span::styled(refresh_text, Style::default().fg(Color::Cyan)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(status, chunks[0]);

    // Right side: Control hints
    let controls = Paragraph::new(Line::from(vec![
        Span::styled(" c ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::styled(" AWS Config ", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(" , ", Style::default().fg(Color::Black).bg(Color::Yellow)),
        Span::styled(" Set ", Style::default().fg(Color::Yellow)),
        Span::raw(" "),
        Span::styled(" ?/h ", Style::default().fg(Color::Black).bg(Color::Cyan)),
        Span::styled(" Help ", Style::default().fg(Color::Cyan)),
        Span::raw(" "),
        Span::styled(" q ", Style::default().fg(Color::Black).bg(Color::Red)),
        Span::styled(" Quit ", Style::default().fg(Color::Red)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(controls, chunks[1]);
}
