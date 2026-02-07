use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::ui::utils::pad_rect;

/// Render home screen - unified panel with responsive layout
pub fn render_home(frame: &mut Frame, app: &App, area: Rect) {
    // Determine layout based on terminal width
    let is_wide = area.width >= 100;
    
    // Render outer block first
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(" Dashboard ")
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(outer_block.clone(), area);
    
    // Get inner area and apply padding for content
    let inner_area = outer_block.inner(area);
    let padded_area = pad_rect(inner_area, 2, 1, 0, 0);
    
    if is_wide {
        // Wide layout: dashboard left, logo+system info right
        // Left side: Dashboard stats
        let dashboard_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Welcome to ", Style::default().fg(Color::White)),
                Span::styled("AWS Cloud Controller", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("📊 Quick Stats", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Region:            ", Style::default().fg(Color::Gray)),
                Span::styled(&app.aws_client.region, Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("   EC2 Instances:     ", Style::default().fg(Color::Gray)),
                Span::styled(app.ec2_instances.len().to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("   Lambda Functions:  ", Style::default().fg(Color::Gray)),
                Span::styled(app.lambda_functions.len().to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("   Auto-stop Timers:  ", Style::default().fg(Color::Gray)),
                Span::styled(app.auto_stop_schedules.len().to_string(), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("💡 Quick Tips", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled("   Press [r] to refresh data from AWS", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("   Press [?] or [h] for keyboard shortcuts", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("   Use [2] to go to EC2 management", Style::default().fg(Color::DarkGray))),
            Line::from(Span::styled("   Use [4] for About & Credits", Style::default().fg(Color::DarkGray))),
        ];
        
        let dashboard = Paragraph::new(dashboard_lines)
            .block(Block::default()) // No borders
            .scroll((app.scroll_offset, 0))
            .wrap(Wrap { trim: true });
        
        // Right side: Logo + System info
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("HOST"))
            .unwrap_or_else(|_| whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()));
        let username = whoami::username();
        let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
        
        let right_lines = vec![
            Line::from(""),
            Line::from(Span::styled(r"   ____  _ ____             ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"  / ___|(_)  _ \ _   _ _ __   ___ ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"  \___ \| | |_) | | | | '_ \ / _ \", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"   ___) | |  _ <| |_| | | | |  __/", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"  |____/|_|_| \_\\__,_|_| |_|\___|", Style::default().fg(Color::Cyan))),
            Line::from(""),
            Line::from(""),
            Line::from(Span::styled("🖥️  System Info", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("   User:    ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}@{}", username, hostname), Style::default().fg(Color::Cyan)),
            ]),
            Line::from(vec![
                Span::styled("   OS:      ", Style::default().fg(Color::Gray)),
                Span::styled(os_info, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(vec![
                Span::styled("   Shell:   ", Style::default().fg(Color::Gray)),
                Span::styled(std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()), Style::default().fg(Color::Green)),
            ]),
            Line::from(vec![
                Span::styled("   Term:    ", Style::default().fg(Color::Gray)),
                Span::styled(std::env::var("TERM").unwrap_or_else(|_| "unknown".to_string()), Style::default().fg(Color::Magenta)),
            ]),
            Line::from(""),
            Line::from(Span::styled("─────────────────────────────", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(vec![
                Span::styled("Made with ", Style::default().fg(Color::Gray)),
                Span::styled("❤️", Style::default().fg(Color::Red)),
                Span::styled(" by ", Style::default().fg(Color::Gray)),
                Span::styled("kingavatar", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        let right_panel = Paragraph::new(right_lines)
            .block(Block::default())
            .scroll((app.scroll_offset, 0)); // No borders
        
        // Split padded inner area - Fixed width for right panel to avoid excessive space
        let inner_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Min(0), Constraint::Length(45)])
            .split(padded_area);
        
        frame.render_widget(dashboard, inner_chunks[0]);
        frame.render_widget(right_panel, inner_chunks[1]);
        
    } else {
        // Narrow layout: stacked vertically
        let hostname = std::env::var("HOSTNAME")
            .or_else(|_| std::env::var("HOST"))
            .unwrap_or_else(|_| whoami::fallible::hostname().unwrap_or_else(|_| "unknown".to_string()));
        let username = whoami::username();
        let os_info = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
        
        let stacked_lines = vec![
            Line::from(""),
            Line::from(vec![
                Span::styled("Welcome to ", Style::default().fg(Color::White)),
                Span::styled("AWS Cloud Controller", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from(Span::styled("📊 Quick Stats", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Region: ", Style::default().fg(Color::Gray)),
                Span::styled(&app.aws_client.region, Style::default().fg(Color::Green)),
                Span::styled("   EC2: ", Style::default().fg(Color::Gray)),
                Span::styled(app.ec2_instances.len().to_string(), Style::default().fg(Color::Yellow)),
                Span::styled("   Lambda: ", Style::default().fg(Color::Gray)),
                Span::styled(app.lambda_functions.len().to_string(), Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(Span::styled("💡 Tips: [r] refresh  [?] help  [2] EC2  [4] About", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled("─────────────────────────────────────────────────", Style::default().fg(Color::DarkGray))),
            Line::from(""),
            Line::from(Span::styled(r"   ____  _ ____                    ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"  / ___|(_)  _ \ _   _ _ __   ___  ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"  \___ \| | |_) | | | | '_ \ / _ \ ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"   ___) | |  _ <| |_| | | | |  __/ ", Style::default().fg(Color::Cyan))),
            Line::from(Span::styled(r"  |____/|_|_| \_\\__,_|_| |_|\___| ", Style::default().fg(Color::Cyan))),
            Line::from(""),
            Line::from(vec![
                Span::styled("   User: ", Style::default().fg(Color::Gray)),
                Span::styled(format!("{}@{}", username, hostname), Style::default().fg(Color::Cyan)),
                Span::styled("   OS: ", Style::default().fg(Color::Gray)),
                Span::styled(os_info, Style::default().fg(Color::Yellow)),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("   Made with ", Style::default().fg(Color::Gray)),
                Span::styled("❤️", Style::default().fg(Color::Red)),
                Span::styled(" by ", Style::default().fg(Color::Gray)),
                Span::styled("kingavatar", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
        ];
        
        let stacked = Paragraph::new(stacked_lines)
            .block(Block::default()) // No borders
            .scroll((app.scroll_offset, 0))
            .wrap(Wrap { trim: true });
        
        frame.render_widget(stacked, padded_area);
    }
}
