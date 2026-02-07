use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::ui::utils::pad_rect;

/// Render Lambda functions screen
pub fn render_lambda(frame: &mut Frame, _app: &App, area: Rect) {
    // Render outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Lambda Functions ")
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(outer_block.clone(), area);
    
    // Get padded inner area
    let inner_area = outer_block.inner(area);
    let padded_area = pad_rect(inner_area, 2, 1, 0, 0);

    let content = Paragraph::new(vec![
        Line::from(""),
        Line::from(Span::styled(
            "   🚧 Lambda Module - Coming Soon! 🚧",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(""),
        Line::from("   Features planned for next version:"),
        Line::from(""),
        Line::from(Span::styled("   • List all Lambda functions", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("   • View function details (runtime, memory, timeout)", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("   • Invoke functions directly", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("   • View recent invocation logs", Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("   Stay tuned! 🎉", Style::default().fg(Color::Cyan))),
    ])
    .block(Block::default()) // No border
    .wrap(Wrap { trim: true });

    frame.render_widget(content, padded_area);
}
