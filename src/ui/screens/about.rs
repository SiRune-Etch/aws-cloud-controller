use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::ui::utils::pad_rect;

/// Render About screen
pub fn render_about(frame: &mut Frame, app: &App, area: Rect) {
    // Render outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" About ")
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(outer_block.clone(), area);
    
    // Get padded inner area
    let inner_area = outer_block.inner(area);
    let padded_area = pad_rect(inner_area, 2, 1, 0, 0);

    // Define content vectors first
    // About content
    let about_content = vec![
        Line::from(""),
        Line::from(Span::styled("AWS Cloud Controller", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled(format!("Version: {}", env!("CARGO_PKG_VERSION")), Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("─────────────────────────────────────────", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled("📖 Description", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("A powerful terminal-based interface for managing", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("AWS cloud resources. Control your EC2 instances,", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("Lambda functions, and more - all from your terminal.", Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("✨ Features", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("• EC2 Instance Management (Start/Stop/Terminate)", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("• Auto-stop Scheduling for Cost Savings", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("• Long-running Instance Alerts with Sound", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("• Lambda Function Management (Coming Soon)", Style::default().fg(Color::Gray))),
        Line::from(Span::styled("• Slack Integration (Planned)", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("─────────────────────────────────────────", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled("🛠️  Built With", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("Rust • Ratatui • AWS SDK • Tokio • Rodio", Style::default().fg(Color::DarkGray))),
    ];

    // Credits content
    let credits_text = vec![
        Line::from(""),
        Line::from(Span::styled(r"   ____  _ ____             ", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(r"  / ___|(_)  _ \ _   _ _ __  ___ ", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(r"  \___ \| | |_) | | | | '_ \/ _ \", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(r"   ___) | |  _ <| |_| | | | |  __/", Style::default().fg(Color::Cyan))),
        Line::from(Span::styled(r"  |____/|_|_| \_\\__,_|_| |_|\___|" , Style::default().fg(Color::Cyan))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("─────────────────────────────", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled("👤 Author", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(vec![
            Span::styled("Made with ", Style::default().fg(Color::Gray)),
            Span::styled("❤️", Style::default().fg(Color::Red)),
            Span::styled(" by", Style::default().fg(Color::Gray)),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("kingavatar ", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            Span::styled("(Saikiran Reddy)", Style::default().fg(Color::Cyan)),
        ]),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("─────────────────────────────", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(Span::styled("🔗 Links", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from(Span::styled("GitHub: github.com/kingavatar", Style::default().fg(Color::DarkGray))),
        Line::from(""),
        Line::from(""),
        Line::from(Span::styled("© 2026 SiRune. All rights reserved.", Style::default().fg(Color::DarkGray))),
    ];

    // Responsive layout based on width
    let is_wide = area.width >= 100;
    
    if is_wide {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
            .split(padded_area);

        // Left: About content
        let about = Paragraph::new(about_content)
            .block(Block::default()) // No border
            .scroll((app.scroll_offset, 0))
            .wrap(Wrap { trim: true });

        frame.render_widget(about, chunks[0]);

        // Right: Credits Board
        let credits_block = Block::default()
            .borders(Borders::ALL)
            .title(" Credits ")
            .border_style(Style::default().fg(Color::Blue));
        
        frame.render_widget(credits_block.clone(), chunks[1]);
        
        let credits_inner = credits_block.inner(chunks[1]);
        let credits_padded = pad_rect(credits_inner, 2, 0, 0, 0); // Left padding: 2
        
        let credits = Paragraph::new(credits_text)
            .block(Block::default())
            .scroll((app.scroll_offset, 0))
            .wrap(Wrap { trim: true });

        frame.render_widget(credits, credits_padded);
    } else {
        // Narrow layout: Combined content
        let mut combined_content = about_content;
        combined_content.push(Line::from(""));
        combined_content.push(Line::from(""));
        combined_content.push(Line::from(Span::styled("────────── CREDITS ──────────", Style::default().fg(Color::Blue))));
        combined_content.extend(credits_text);
        
        let content = Paragraph::new(combined_content)
            .block(Block::default()) // No border
            .scroll((app.scroll_offset, 0))
            .wrap(Wrap { trim: true });
            
        frame.render_widget(content, padded_area);
    }
}
