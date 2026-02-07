use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::{App, Dialog};
use crate::settings::SettingsField;
use crate::ui::utils::{centered_rect, pad_rect};

/// Render dialog overlay
pub fn render_dialog(frame: &mut Frame, app: &App) {
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
        Dialog::SessionExpired => {
            let mut expired_content = vec![
                Line::from(""),
                Line::from(Span::styled("⚠️  AWS Session Token Expired", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("Select AWS Profile to refresh (Use ↑/↓, [Enter] Activate):"),
                Line::from(""),
            ];
            
            if app.available_profiles.is_empty() {
                expired_content.push(Line::from(Span::styled("No profiles found in ~/.aws/config", Style::default().fg(Color::Red))));
            } else {
                for (i, profile) in app.available_profiles.iter().enumerate() {
                    let is_selected = i == app.selected_profile_index;
                    let is_active = app.active_profile_name.as_ref().map(|p| p == profile).unwrap_or(false);

                    let style = if is_selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if is_active {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    let prefix = if is_selected { " > " } else { "   " };
                    let status_icon = if is_active { " ✅" } else { "" };
                    
                    expired_content.push(Line::from(Span::styled(format!("{}{}{}", prefix, profile, status_icon), style)));
                }
            }
            
            expired_content.extend_from_slice(&[
                Line::from(""),
                Line::from(Span::styled("Quick Fix:", Style::default().fg(Color::Green))),
                Line::from("1. Select your profile above"),
                Line::from("2. Press 'l' (L) to launch browser login"),
                Line::from("3. After login, press 'r' to retry"),
                Line::from(""),
            ]);
            
            expired_content.push(Line::from(vec![
                Span::raw("          "),
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Activate   "),
                Span::styled("[Esc]", Style::default().fg(Color::Green)),
                Span::raw(" Dismiss   "),
                Span::styled("[l]", Style::default().fg(Color::Yellow)),
                Span::raw(" SSO Login   "),
                Span::styled("[r]", Style::default().fg(Color::Cyan)),
                Span::raw(" Retry"),
            ]));
            
            ((60, 60), " 🔑 Session Expired ", expired_content, Style::default().fg(Color::Red))
        }
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
                    Span::raw("          "),
                    Span::styled("[Enter/Esc]", Style::default().fg(Color::Green)),
                    Span::raw(" Dismiss   "),
                    Span::styled("[l]", Style::default().fg(Color::Yellow)),
                    Span::raw(" SSO Login"),
                ]),
            ];
            ((70, 70), " 🔧 AWS Setup Required ", setup_content, Style::default().fg(Color::Yellow))
        }
        Dialog::ConfigureAws => {
            let mut config_content = vec![
                Line::from(""),
                Line::from(Span::styled("🔧 AWS Configuration Options", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from("Select AWS Profile (Use ↑/↓, [Enter] to Activate):"),
                Line::from(""),
            ];
            
            if app.available_profiles.is_empty() {
                config_content.push(Line::from(Span::styled("No profiles found in ~/.aws/config", Style::default().fg(Color::Red))));
            } else {
                for (i, profile) in app.available_profiles.iter().enumerate() {
                    let is_selected = i == app.selected_profile_index;
                    let is_active = app.active_profile_name.as_ref().map(|p| p == profile).unwrap_or(false);
                    
                    let style = if is_selected {
                        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                    } else if is_active {
                        Style::default().fg(Color::Green)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    
                    let prefix = if is_selected { " > " } else { "   " };
                    let status_icon = if is_active { " ✅" } else { "" };
                    
                    config_content.push(Line::from(Span::styled(format!("{}{}{}", prefix, profile, status_icon), style)));
                }
            }
            
            config_content.extend_from_slice(&[
                Line::from(""),
                Line::from(Span::styled("Manual Options:", Style::default().fg(Color::DarkGray))),
                Line::from("  Option 2: Assume Role (export AWS_PROFILE=...)"),
                Line::from("  Option 3: Env Vars (AWS_ACCESS_KEY_ID, ...)" ),
                Line::from(""),
                Line::from(Span::styled("Current Status:", Style::default().fg(Color::DarkGray))),
                Line::from(format!("  AWS Region: {}", app.config.aws_region.clone().unwrap_or_else(|| "default".to_string()))),
                Line::from(""),
            ]);
            
            config_content.push(Line::from(vec![
                Span::raw("          "),
                Span::styled("[Enter]", Style::default().fg(Color::Green)),
                Span::raw(" Activate   "),
                Span::styled("[Esc]", Style::default().fg(Color::Green)),
                Span::raw(" Dismiss   "),
                Span::styled("[l]", Style::default().fg(Color::Yellow)),
                Span::raw(" SSO Login"),
            ]));
            
            ((60, 50), " ☁️  AWS Configuration ", config_content, Style::default().fg(Color::Cyan))
        }
        Dialog::Settings => {
            // Get the draft settings to display (or current if no draft)
            let settings = app.settings_draft.as_ref().unwrap_or(&app.settings);
            
            // Helper to create a row with highlight if selected
            let make_row = |name: &str, value: &str, field: SettingsField| -> Line {
                let is_selected = app.settings_selected_field == field;
                let name_style = if is_selected {
                    Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };
                let value_style = if is_selected {
                    Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::Green)
                };
                let arrow = if is_selected { "▶ " } else { "  " };
                
                Line::from(vec![
                    Span::styled(arrow, Style::default().fg(Color::Yellow)),
                    Span::styled(format!("{:20}", name), name_style),
                    Span::styled(format!("< {} >", value), value_style),
                ])
            };
            
            let settings_content = vec![
                Line::from(""),
                Line::from(Span::styled("⚙️  Application Settings", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))),
                Line::from(""),
                Line::from(Span::styled("Use ↑/↓ to navigate, ←/→ to change values", Style::default().fg(Color::DarkGray))),
                Line::from(""),
                make_row("Refresh Interval", &settings.format_refresh_interval(), SettingsField::RefreshInterval),
                Line::from(""),
                make_row("Show Logs Panel", if settings.show_logs_panel { "Yes" } else { "No" }, SettingsField::ShowLogsPanel),
                Line::from(""),
                make_row("Log Verbosity", &settings.format_log_level(), SettingsField::LogLevel),
                Line::from(""),
                make_row("Alert Threshold", &settings.format_alert_threshold(), SettingsField::AlertThreshold),
                Line::from(""),
                make_row("Sound Alerts", if settings.sound_enabled { "On" } else { "Off" }, SettingsField::SoundEnabled),
                Line::from(""),
                make_row("Test Alert Sound", "[ Press Enter ]", SettingsField::TestSound),
                Line::from(""),
                Line::from(""),
                Line::from(vec![
                    Span::styled("[Enter]", Style::default().fg(Color::Green)),
                    Span::raw(" Save   "),
                    Span::styled("[Esc]", Style::default().fg(Color::Red)),
                    Span::raw(" Cancel"),
                ]),
            ];
            ((50, 60), " ⚙️  Settings ", settings_content, Style::default().fg(Color::Magenta))
        }
        Dialog::Changelog => {
             let changelog_text = include_str!("../../../CHANGELOG.md");
             let content: Vec<Line> = changelog_text.lines()
                 .map(|l: &str| {
                     if l.starts_with("# ") {
                         Line::from(Span::styled(l, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD|Modifier::UNDERLINED)))
                     } else if l.starts_with("## ") {
                         Line::from(Span::styled(l, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
                     } else if l.starts_with("### ") {
                         Line::from(Span::styled(l, Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)))
                     } else {
                         // Default text
                         Line::from(Span::raw(l))
                     }
                 })
                 .collect();
             
             let mut final_content = vec![
                 Line::from(""),
                 Line::from(vec![
                     Span::styled("[Esc/Enter]", Style::default().fg(Color::Green)),
                     Span::raw(" Close   "),
                     Span::styled("[↑/↓]", Style::default().fg(Color::Yellow)),
                     Span::raw(" Scroll"),
                 ]),
                 Line::from(""),
                 Line::from(Span::styled("─────────────────────────────────────────", Style::default().fg(Color::DarkGray))),
                 Line::from(""),
             ];
             final_content.extend(content);
             
             ((70, 80), " 📜 Changelog ", final_content, Style::default().fg(Color::Cyan))
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

pub fn draw_loading_overlay(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(" ⏳ Processing ");
        
    let area = centered_rect(40, 20, area);
    f.render_widget(Clear, area); // Clear background
    f.render_widget(block, area);

    let text = vec![
        Line::from(""),
        Line::from(Span::styled(app.status_message.as_str(), Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))),
        Line::from(""),
        Line::from("Please wait..."),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(ratatui::layout::Alignment::Center)
        .wrap(Wrap { trim: true });
        
    // Inner area for text
    let inner_area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(1), Constraint::Length(1)].as_ref())
        .split(area)[1];
        
    f.render_widget(paragraph, inner_area);
}
