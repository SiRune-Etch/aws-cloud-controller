//! UI rendering with Ratatui

pub mod components;
pub mod screens;
pub mod utils;

use ratatui::{
    layout::{Constraint, Direction, Layout},
    Frame,
};

use crate::app::{App, Dialog, Screen};
use self::components::{
    dialogs::{draw_loading_overlay, render_dialog},
    statusbar::render_status_bar,
    toast::render_toasts,
};
use self::screens::{
    about::render_about,
    ec2::render_ec2,
    home::render_home,
    lambda::render_lambda,
    logs::render_logs,
};

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
    
    // Render toasts on top
    render_toasts(frame, app);
    
    // Render loading overlay over everything else if active
    if app.is_loading {
        draw_loading_overlay(frame, app, frame.area());
    }
}

use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Tabs},
};

/// Render navigation tabs
fn render_tabs(frame: &mut Frame, app: &App, area: Rect) {
    let mut titles = vec!["🏠 Home [1]", "💻 EC2 [2]", "λ Lambda [3]", "ℹ️ About [4]"];
    
    // Only show Logs tab if enabled (always last)
    if app.settings.show_logs_panel {
        titles.push("📋 Logs [5]");
    }
    
    let selected_idx = match app.current_screen {
        Screen::Home => 0,
        Screen::Ec2 => 1,
        Screen::Lambda => 2,
        Screen::About => 3,
        Screen::Logs => if app.settings.show_logs_panel { 4 } else { 0 },
    };
    
    let tabs = Tabs::new(titles)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" AWS Cloud Controller ")
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .select(selected_idx)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        );

    frame.render_widget(tabs, area);
}

/// Render main content based on current screen
fn render_content(frame: &mut Frame, app: &App, area: Rect) {
    match app.current_screen {
        Screen::Home => render_home(frame, app, area),
        Screen::Ec2 => render_ec2(frame, app, area),
        Screen::Lambda => render_lambda(frame, app, area),
        Screen::Logs => render_logs(frame, app, area),
        Screen::About => render_about(frame, app, area),
    }
}
