use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, ToastType};

/// Render toast notifications in top-right corner
pub fn render_toasts(frame: &mut Frame, app: &App) {
    if app.toasts.is_empty() {
        return;
    }
    
    let area = frame.area();
    let max_toast_width = 50;
    let toast_height = 3;
    
    // Stack toasts from top to bottom
    for (idx, toast) in app.toasts.iter().rev().take(3).enumerate() {
        let y_offset = (idx as u16 * (toast_height + 1)) + 1;
        
        if y_offset + toast_height > area.height {
            break; // Don't render if it would go off screen
        }
        
        let toast_area = Rect {
            x: area.width.saturating_sub(max_toast_width + 2),
            y: area.y + y_offset,
            width: max_toast_width.min(area.width),
            height: toast_height,
        };
        
        let (border_color, icon) = match toast.toast_type {
            ToastType::Success => (Color::Green, "✓"),
            ToastType::Error => (Color::Red, "✗"),
            ToastType::Info => (Color::Cyan, "ℹ"),
        };
        
        let toast_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(border_color))
            .style(Style::default().bg(Color::Black));
        
        frame.render_widget(Clear, toast_area);
        frame.render_widget(toast_block.clone(), toast_area);
        
        let inner = toast_block.inner(toast_area);
        let text = Paragraph::new(Line::from(vec![
            Span::styled(format!("{} ", icon), Style::default().fg(border_color).add_modifier(Modifier::BOLD)),
            Span::styled(&toast.message, Style::default().fg(Color::White)),
        ]))
        .wrap(Wrap { trim: true });
        
        frame.render_widget(text, inner);
    }
}
