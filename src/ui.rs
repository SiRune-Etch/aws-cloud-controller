//! UI rendering with Ratatui

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs, Wrap},
    Frame,
};

use crate::app::{App, Dialog, Screen};

/// Main render function
pub fn render(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(1),    // Content
            Constraint::Length(3), // Status bar
        ])
        .split(frame.area());

    render_tabs(frame, app, chunks[0]);
    render_content(frame, app, chunks[1]);
    render_status_bar(frame, app, chunks[2]);

    // Render dialog overlay if present
    if app.dialog != Dialog::None {
        render_dialog(frame, app);
    }
}

/// Render navigation tabs
fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let titles = vec!["🏠 Home [1]", "💻 EC2 [2]", "λ Lambda [3]", "ℹ️ About [4]"];
    
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" AWS Cloud Controller ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .select(match app.current_screen {
            Screen::Home => 0,
            Screen::Ec2 => 1,
            Screen::Lambda => 2,
            Screen::About => 3,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

/// Helper to pad an area
fn pad_rect(area: Rect, left: u16, right: u16, top: u16, bottom: u16) -> Rect {
    let new_x = area.x.saturating_add(left);
    let new_y = area.y.saturating_add(top);
    let new_width = area.width.saturating_sub(left + right);
    let new_height = area.height.saturating_sub(top + bottom);
    
    Rect {
        x: new_x,
        y: new_y,
        width: new_width,
        height: new_height,
    }
}

/// Render main content based on current screen
fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.current_screen {
        Screen::Home => render_home(frame, app, area),
        Screen::Ec2 => render_ec2(frame, app, area),
        Screen::Lambda => render_lambda(frame, app, area),
        Screen::About => render_about(frame, app, area),
    }
}

/// Render home screen - unified panel with responsive layout
fn render_home(frame: &mut Frame, app: &App, area: Rect) {
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
    
    // Welcome header content
    // ... constructs content ...
    
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


/// Render About screen
fn render_about(frame: &mut Frame, app: &App, area: Rect) {
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
        Line::from(Span::styled("Version: 0.1.0", Style::default().fg(Color::Yellow))),
        Line::from(Span::styled("License: MIT", Style::default().fg(Color::DarkGray))),
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
/// Render EC2 instances screen
fn render_ec2(frame: &mut Frame, app: &App, area: Rect) {
    // Render outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(format!(" EC2 Instances ({}) ", app.ec2_instances.len()))
        .border_style(Style::default().fg(Color::Blue));
    frame.render_widget(outer_block.clone(), area);
    
    // Get padded inner area
    let inner_area = outer_block.inner(area);
    let padded_area = pad_rect(inner_area, 2, 1, 1, 0);

    if app.ec2_instances.is_empty() {
        let msg = Paragraph::new(vec![
            Line::from("No EC2 instances loaded."),
            Line::from(""),
            Line::from(Span::styled("Press [r] to refresh", Style::default().fg(Color::Yellow))),
        ])
        .block(Block::default()); // No border
        frame.render_widget(msg, padded_area);
        return;
    }

    // Create table rows
    let rows: Vec<Row> = app
        .ec2_instances
        .iter()
        .enumerate()
        .map(|(i, instance)| {
            let state_style = match instance.state.as_str() {
                "running" => Style::default().fg(Color::Green),
                "stopped" => Style::default().fg(Color::Red),
                "pending" | "stopping" => Style::default().fg(Color::Yellow),
                _ => Style::default().fg(Color::Gray),
            };

            let selected_style = if i == app.ec2_selected {
                Style::default().bg(Color::DarkGray).add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            // Check if this instance has auto-stop scheduled
            let has_schedule = app.auto_stop_schedules.iter().any(|(id, _)| *id == instance.id);
            let schedule_indicator = if has_schedule { "⏰" } else { "" };

            Row::new(vec![
                Cell::from(if i == app.ec2_selected { "▶" } else { " " }),
                Cell::from(instance.name.clone()),
                Cell::from(instance.id.clone()),
                Cell::from(instance.instance_type.clone()),
                Cell::from(Span::styled(instance.state.clone(), state_style)),
                Cell::from(instance.public_ip.clone().unwrap_or_else(|| "-".to_string())),
                Cell::from(schedule_indicator),
            ])
            .style(selected_style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(2),  // Selector
            Constraint::Min(20),    // Name
            Constraint::Length(20), // ID
            Constraint::Length(12), // Type
            Constraint::Length(12), // State
            Constraint::Length(16), // IP
            Constraint::Length(3),  // Schedule
        ],
    )
    .header(
        Row::new(vec!["", "Name", "Instance ID", "Type", "State", "Public IP", "⏰"])
            .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
            .bottom_margin(1),
    )
    .block(Block::default()); // No border

    // Use stateful widget for scrolling
    let mut state = app.ec2_table_state.clone();
    frame.render_stateful_widget(table, padded_area, &mut state);
}

/// Render Lambda functions screen
fn render_lambda(frame: &mut Frame, _app: &App, area: Rect) {
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

/// Render status bar with control hints
fn render_status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(40), Constraint::Length(35)])
        .split(area);

    // Left side: Status message
    let loading_indicator = if app.is_loading { "⏳ " } else { "" };
    let alert_count = if app.pending_alerts.is_empty() {
        String::new()
    } else {
        format!(" | 🔔 {} alerts", app.pending_alerts.len())
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(loading_indicator, Style::default().fg(Color::Yellow)),
        Span::styled(&app.status_message, Style::default().fg(Color::White)),
        Span::styled(
            format!(" | Region: {}", app.aws_client.region),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled(alert_count, Style::default().fg(Color::Red)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    frame.render_widget(status, chunks[0]);

    // Right side: Control hints
    let controls = Paragraph::new(Line::from(vec![
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

/// Render dialog overlay
fn render_dialog(frame: &mut Frame, app: &App) {
    let (area_size, title, content, style) = match &app.dialog {
        Dialog::Help => {
            let help_content = vec![
                Line::from(""),
                Line::from(Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![Span::styled("  1/2/3     ", Style::default().fg(Color::Yellow)), Span::raw("Switch tabs (Home/EC2/Lambda)")]),
                Line::from(vec![Span::styled("  ↑/↓ j/k   ", Style::default().fg(Color::Yellow)), Span::raw("Navigate list")]),
                Line::from(vec![Span::styled("  Enter     ", Style::default().fg(Color::Yellow)), Span::raw("Select / Confirm")]),
                Line::from(vec![Span::styled("  r         ", Style::default().fg(Color::Yellow)), Span::raw("Refresh data")]),
                Line::from(""),
                Line::from(Span::styled("EC2 Controls", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![Span::styled("  s         ", Style::default().fg(Color::Yellow)), Span::raw("Start instance")]),
                Line::from(vec![Span::styled("  x         ", Style::default().fg(Color::Yellow)), Span::raw("Stop instance")]),
                Line::from(vec![Span::styled("  t         ", Style::default().fg(Color::Yellow)), Span::raw("Terminate instance")]),
                Line::from(vec![Span::styled("  a         ", Style::default().fg(Color::Yellow)), Span::raw("Schedule auto-stop (1 hour)")]),
                Line::from(""),
                Line::from(Span::styled("General", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(vec![Span::styled("  ?/h       ", Style::default().fg(Color::Yellow)), Span::raw("Show this help")]),
                Line::from(vec![Span::styled("  q         ", Style::default().fg(Color::Red)), Span::raw("Quit application")]),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::raw("          "),
                    Span::styled("[Enter/q/Esc]", Style::default().fg(Color::Green)),
                    Span::raw(" Close"),
                ]),
            ];
            ((60, 60), " ⌨️  Keyboard Shortcuts ", help_content, Style::default().fg(Color::Cyan))
        }
        Dialog::ConfirmTerminate(id) => (
            (50, 30),
            " ⚠️  Confirm Termination ",
            vec![
                Line::from(""),
                Line::from(Span::styled(
                    "Are you sure you want to TERMINATE this instance?",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::raw("Instance: "),
                    Span::styled(id.clone(), Style::default().fg(Color::Yellow)),
                ]),
                Line::from(""),
                Line::from(Span::styled(
                    "This action is IRREVERSIBLE!",
                    Style::default().fg(Color::Red),
                )),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Confirm   "),
                    Span::styled("[q/Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]),
            ],
            Style::default().fg(Color::Red),
        ),
        Dialog::ScheduleAutoStop(id) => (
            (50, 30),
            " ⏰ Schedule Auto-Stop ",
            vec![
                Line::from(""),
                Line::from(format!("Instance: {}", id)),
                Line::from(""),
                Line::from("Default: Stop in 1 hour"),
                Line::from(""),
                Line::from(Span::styled(
                    "(Custom durations coming soon)",
                    Style::default().fg(Color::DarkGray),
                )),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Schedule   "),
                    Span::styled("[q/Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]),
            ],
            Style::default().fg(Color::Cyan),
        ),
        Dialog::Alert(msg) => (
            (50, 25),
            " 🔔 Alert ",
            vec![
                Line::from(""),
                Line::from(Span::styled(msg.clone(), Style::default().fg(Color::Yellow))),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter/q]", Style::default().fg(Color::Green)),
                    Span::raw(" Dismiss"),
                ]),
            ],
            Style::default().fg(Color::Yellow),
        ),
        Dialog::Setup => {
            let setup_content = vec![
                Line::from(""),
                Line::from(Span::styled("⚙️  AWS Credentials Not Found", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled("To use AWS Cloud Controller, you need to configure ", Style::default().fg(Color::White))),
                Line::from(Span::styled("your AWS credentials. Choose one of these methods:", Style::default().fg(Color::White))),
                Line::from(""),
                Line::from(Span::styled("Option 1: AWS CLI Configuration", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Run: aws configure", Style::default().fg(Color::Green))),
                Line::from(""),
                Line::from(Span::styled("Option 2: Environment Variables", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  export AWS_ACCESS_KEY_ID=<your-key>", Style::default().fg(Color::Green))),
                Line::from(Span::styled("  export AWS_SECRET_ACCESS_KEY=<your-secret>", Style::default().fg(Color::Green))),
                Line::from(Span::styled("  export AWS_DEFAULT_REGION=us-east-1", Style::default().fg(Color::Green))),
                Line::from(""),
                Line::from(Span::styled("Option 3: AWS SSO", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(Span::styled("  Run: aws sso login --profile <profile>", Style::default().fg(Color::Green))),
                Line::from(""),
                Line::from(""),
                Line::from(Span::styled("After configuring, restart the application.", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                Line::from(vec![
                    Span::raw("            "),
                    Span::styled("[Enter/Esc]", Style::default().fg(Color::Green)),
                    Span::raw(" Dismiss"),
                ]),
            ];
            ((70, 70), " 🔧 AWS Setup Required ", setup_content, Style::default().fg(Color::Yellow))
        }
        Dialog::None => return,
    };


    let area = centered_rect(area_size.0, area_size.1, frame.area());
    
    // Clear background
    frame.render_widget(Clear, area);
    
    // Render outer block
    let outer_block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(style);
    frame.render_widget(outer_block.clone(), area);

    // Get padded inner area
    let inner_area = outer_block.inner(area);
    let padded_area = pad_rect(inner_area, 2, 1, 0, 0);

    let dialog = Paragraph::new(content)
        .block(Block::default()) // No border
        .scroll((app.dialog_scroll_offset, 0))
        .wrap(Wrap { trim: true });

    frame.render_widget(dialog, padded_area);
}

/// Helper to create a centered rect
fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
