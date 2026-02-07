use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

use crate::app::App;
use crate::ui::utils::pad_rect;

/// Render EC2 instances screen
pub fn render_ec2(frame: &mut Frame, app: &App, area: Rect) {
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
