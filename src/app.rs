//! Application state and core logic

use std::time::Duration;

use anyhow::Result;
use chrono::{DateTime, Utc};
use ratatui::backend::Backend;
use ratatui::Terminal;
use ratatui::widgets::TableState;
use rodio::Source;


use crate::aws::{AwsClient, Ec2Instance, LambdaFunction};
use crate::config::AppConfig;
use crate::event::{poll_event, AppEvent};
use crate::ui;

/// Current screen/tab
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Home,
    Ec2,
    Lambda,
    About,
}

/// Dialog/modal state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    None,
    Help,                     // Help popup
    Setup,                    // AWS setup instructions
    ConfirmTerminate(String), // instance_id
    ScheduleAutoStop(String), // instance_id  
    Alert(String),            // message
}

/// Application state
pub struct App {
    // Core
    pub config: AppConfig,
    pub aws_client: AwsClient,
    pub should_quit: bool,
    pub current_screen: Screen,
    #[allow(dead_code)] // Used for Setup dialog state
    pub aws_configured: bool,
    
    // Status
    pub status_message: String,
    pub is_loading: bool,
    pub scroll_offset: u16,
    
    // EC2 State
    pub ec2_instances: Vec<Ec2Instance>,
    pub ec2_selected: usize,
    pub ec2_table_state: TableState,
    pub auto_stop_schedules: Vec<(String, DateTime<Utc>)>, // (instance_id, stop_time)
    
    // Lambda State
    pub lambda_functions: Vec<LambdaFunction>,
    pub lambda_selected: usize,
    
    // Dialogs
    pub dialog: Dialog,
    
    // Alerts
    pub pending_alerts: Vec<String>,
    pub last_alert_check: Option<DateTime<Utc>>,
    
    // Window state
    pub window_size: (u16, u16),
    pub dialog_scroll_offset: u16,
}


impl App {
    /// Create a new application instance
    pub async fn new() -> Result<Self> {
        let config = AppConfig::default();
        let aws_client = AwsClient::new(config.aws_region.as_deref()).await?;
        
        // Check if AWS credentials are configured by trying a simple operation
        let aws_configured = Self::check_aws_credentials().await;
        let initial_dialog = if aws_configured {
            Dialog::None
        } else {
            Dialog::Setup
        };
        
        Ok(Self {
            config,
            aws_client,
            should_quit: false,
            current_screen: Screen::Home,
            aws_configured,
            status_message: if aws_configured { "Ready".to_string() } else { "AWS credentials not configured".to_string() },
            is_loading: false,
            ec2_instances: Vec::new(),
            ec2_selected: 0,
            ec2_table_state: TableState::default(),
            auto_stop_schedules: Vec::new(),
            lambda_functions: Vec::new(),
            lambda_selected: 0,
            dialog: initial_dialog,
            pending_alerts: Vec::new(),
            last_alert_check: None,
            scroll_offset: 0,
            window_size: (80, 24), // Default, will be updated
            dialog_scroll_offset: 0,
        })
    }
    
    /// Check if AWS credentials are configured
    async fn check_aws_credentials() -> bool {
        // Check for AWS credentials by looking at environment variables or credentials file
        if std::env::var("AWS_ACCESS_KEY_ID").is_ok() && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok() {
            return true;
        }
        
        // Check for AWS credentials file
        if let Some(home) = dirs::home_dir() {
            let creds_path = home.join(".aws").join("credentials");
            if creds_path.exists() {
                return true;
            }
        }
        
        // Check for AWS config file with SSO
        if let Some(home) = dirs::home_dir() {
            let config_path = home.join(".aws").join("config");
            if config_path.exists() {
                return true;
            }
        }
        
        false
    }

    /// Main event loop
    pub async fn run<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<()> {
        let tick_rate = Duration::from_millis(self.config.tick_rate_ms);

        loop {
            // Render UI
            terminal.draw(|f| ui::render(f, self))?;

            // Handle events
            if let Some(event) = poll_event(tick_rate)? {
                self.handle_event(event).await?;
            }

            // Sync table state with selection
            if self.current_screen == Screen::Ec2 {
                if self.ec2_instances.is_empty() {
                    self.ec2_table_state.select(None);
                } else {
                    self.ec2_table_state.select(Some(self.ec2_selected));
                }
            }

            // Check for alerts periodically
            self.check_alerts();

            if self.should_quit {
                break;
            }
        }

        Ok(())
    }

    /// Handle application events
    async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
        // Handle dialog events first
        if self.dialog != Dialog::None {
            return self.handle_dialog_event(event).await;
        }

        match event {
            AppEvent::Quit => self.should_quit = true,
            
            AppEvent::NavigateTab(idx) => {
                let new_screen = match idx {
                    0 => Screen::Home,
                    1 => Screen::Ec2,
                    2 => Screen::Lambda,
                    3 => Screen::About,
                    _ => self.current_screen,
                };
                
                if new_screen != self.current_screen {
                    self.current_screen = new_screen;
                    self.scroll_offset = 0;
                }
            }
            
            AppEvent::Up => self.move_selection(-1),
            AppEvent::Down => self.move_selection(1),
            
            AppEvent::Refresh => self.refresh_data().await?,
            
            AppEvent::Start => self.start_selected_instance().await?,
            AppEvent::Stop => self.stop_selected_instance().await?,
            AppEvent::Terminate => self.confirm_terminate_instance()?,
            AppEvent::Schedule => self.open_schedule_dialog()?,
            AppEvent::ShowHelp => {
                self.dialog = Dialog::Help;
                self.dialog_scroll_offset = 0;
            }
            
            AppEvent::Enter => self.handle_enter().await?,
            
            AppEvent::Resize(w, h) => {
                self.window_size = (w, h);
            }
            
            AppEvent::None => {}
        }

        Ok(())
    }

    /// Handle events when dialog is open
    async fn handle_dialog_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => self.dialog = Dialog::None,
            AppEvent::Up => {
                self.dialog_scroll_offset = self.dialog_scroll_offset.saturating_sub(1);
            }
            AppEvent::Down => {
                // Calculate max scroll for current dialog based on window size
                let (_, h) = self.window_size;
                
                let (percent_y, content_lines): (u16, u16) = match self.dialog {
                    Dialog::Setup => (70, 27),
                    Dialog::Help => (60, 27),
                    Dialog::ConfirmTerminate(_) => (30, 12),
                    Dialog::ScheduleAutoStop(_) => (30, 12),
                    Dialog::Alert(_) => (25, 10),
                    Dialog::None => (0, 0),
                };
                
                let chunk_height = h * percent_y / 100;
                // Subtract 2 for borders
                let available_height = chunk_height.saturating_sub(2);
                
                let max_scroll = content_lines.saturating_sub(available_height);
                
                if self.dialog_scroll_offset < max_scroll {
                     self.dialog_scroll_offset += 1;
                }
            }
            AppEvent::Enter => {
                match &self.dialog {
                    Dialog::ConfirmTerminate(id) => {
                        let id = id.clone();
                        self.dialog = Dialog::None;
                        self.terminate_instance(&id).await?;
                    }
                    Dialog::ScheduleAutoStop(id) => {
                        let id = id.clone();
                        self.dialog = Dialog::None;
                        self.schedule_auto_stop(&id, Duration::from_secs(3600))?; // Default 1 hour
                    }
                    Dialog::Alert(_) | Dialog::Help | Dialog::Setup => {
                        self.dialog = Dialog::None;
                    }
                    Dialog::None => {}
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Move selection up or down
    fn move_selection(&mut self, delta: i32) {
        match self.current_screen {
            Screen::Ec2 => {
                let len = self.ec2_instances.len();
                if len > 0 {
                    self.ec2_selected = ((self.ec2_selected as i32 + delta).rem_euclid(len as i32)) as usize;
                }
            }
            Screen::Lambda => {
                let len = self.lambda_functions.len();
                if len > 0 {
                    self.lambda_selected = ((self.lambda_selected as i32 + delta).rem_euclid(len as i32)) as usize;
                }
            }
            Screen::Home | Screen::About => {
                let (w, h) = self.window_size;
                // Estimate available height (minus headers, borders, footer)
                // Tabs(3) + Status(3) + Borders(2) = 8.
                let available_height = h.saturating_sub(8);
                
                let content_height: u16 = match self.current_screen {
                    // Home content height depends on width (wide vs narrow layout)
                    Screen::Home => if w >= 100 { 18 } else { 25 },
                    // About content height also depends on width (side-by-side vs stacked)
                    Screen::About => if w >= 100 { 30 } else { 58 },
                    _ => 0,
                };
                
                // Allow scrolling only if content exceeds available height
                let max_scroll = content_height.saturating_sub(available_height);
                
                if delta > 0 {
                    self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
                } else {
                    self.scroll_offset = self.scroll_offset.saturating_sub(1);
                }
            }
        }
    }

    /// Refresh data from AWS
    pub async fn refresh_data(&mut self) -> Result<()> {
        self.is_loading = true;
        self.status_message = "Loading...".to_string();

        match self.current_screen {
            Screen::Ec2 | Screen::Home => {
                match self.aws_client.list_ec2_instances().await {
                    Ok(instances) => {
                        self.ec2_instances = instances;
                        self.status_message = format!("Loaded {} EC2 instances", self.ec2_instances.len());
                    }
                    Err(e) => {
                        self.status_message = format!("Error: {}", e);
                    }
                }
            }
            Screen::Lambda => {
                match self.aws_client.list_lambda_functions().await {
                    Ok(functions) => {
                        self.lambda_functions = functions;
                        self.status_message = format!("Loaded {} Lambda functions", self.lambda_functions.len());
                    }
                    Err(e) => {
                        self.status_message = format!("Error: {}", e);
                    }
                }
            }
            Screen::About => {
                // About screen is static, nothing to refresh
                self.status_message = "About screen - no data to refresh".to_string();
            }
        }

        self.is_loading = false;
        Ok(())
    }

    /// Start the selected EC2 instance
    async fn start_selected_instance(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            let id = instance.id.clone();
            self.status_message = format!("Starting {}...", id);
            
            if let Err(e) = self.aws_client.start_instance(&id).await {
                self.status_message = format!("Failed to start: {}", e);
            } else {
                self.status_message = format!("Started {}", id);
                self.refresh_data().await?;
            }
        }
        Ok(())
    }

    /// Stop the selected EC2 instance
    async fn stop_selected_instance(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            let id = instance.id.clone();
            self.status_message = format!("Stopping {}...", id);
            
            if let Err(e) = self.aws_client.stop_instance(&id).await {
                self.status_message = format!("Failed to stop: {}", e);
            } else {
                self.status_message = format!("Stopped {}", id);
                self.refresh_data().await?;
            }
        }
        Ok(())
    }

    /// Confirm termination dialog
    fn confirm_terminate_instance(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            self.dialog = Dialog::ConfirmTerminate(instance.id.clone());
            self.dialog_scroll_offset = 0;
        }
        Ok(())
    }

    /// Terminate an EC2 instance
    async fn terminate_instance(&mut self, instance_id: &str) -> Result<()> {
        self.status_message = format!("Terminating {}...", instance_id);
        
        if let Err(e) = self.aws_client.terminate_instance(instance_id).await {
            self.status_message = format!("Failed to terminate: {}", e);
        } else {
            self.status_message = format!("Terminated {}", instance_id);
            self.refresh_data().await?;
        }
        Ok(())
    }

    /// Open auto-stop scheduling dialog
    fn open_schedule_dialog(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            self.dialog = Dialog::ScheduleAutoStop(instance.id.clone());
            self.dialog_scroll_offset = 0;
        }
        Ok(())
    }

    /// Schedule auto-stop for an instance
    fn schedule_auto_stop(&mut self, instance_id: &str, duration: Duration) -> Result<()> {
        let stop_time = Utc::now() + chrono::Duration::from_std(duration)?;
        
        // Remove existing schedule for this instance
        self.auto_stop_schedules.retain(|(id, _)| id != instance_id);
        
        // Add new schedule
        self.auto_stop_schedules.push((instance_id.to_string(), stop_time));
        self.status_message = format!("Scheduled auto-stop for {} at {}", instance_id, stop_time.format("%H:%M:%S"));
        
        Ok(())
    }

    /// Handle Enter key based on current screen
    async fn handle_enter(&mut self) -> Result<()> {
        match self.current_screen {
            Screen::Home => {
                // Could navigate to details or do nothing
            }
            Screen::Ec2 => {
                // Toggle instance details or perform default action
                self.refresh_data().await?;
            }
            Screen::Lambda => {
                // Invoke selected lambda (for now, just log)
                if let Some(func) = self.lambda_functions.get(self.lambda_selected) {
                    self.status_message = format!("Lambda invocation coming soon: {}", func.name);
                }
            }
            Screen::About => {
                // About screen has no interactive elements
            }
        }
        Ok(())
    }

    /// Check for instances running too long without auto-stop
    fn check_alerts(&mut self) {
        let now = Utc::now();
        
        // Only check every 30 seconds
        if let Some(last_check) = self.last_alert_check {
            if now.signed_duration_since(last_check).num_seconds() < 30 {
                return;
            }
        }
        self.last_alert_check = Some(now);

        let threshold = chrono::Duration::from_std(self.config.alerts.alert_threshold).unwrap_or(chrono::Duration::hours(1));

        for instance in &self.ec2_instances {
            if instance.state == "running" {
                // Check if this instance has an auto-stop scheduled
                let has_schedule = self.auto_stop_schedules.iter().any(|(id, _)| *id == instance.id);
                
                if !has_schedule {
                    if let Some(launch_time) = instance.launch_time {
                        let running_duration = now.signed_duration_since(launch_time);
                        
                        if running_duration > threshold {
                            let alert_msg = format!(
                                "⚠️ Instance {} ({}) running for {} without auto-stop!",
                                instance.name,
                                instance.id,
                                format_duration(running_duration)
                            );
                            
                            // Avoid duplicate alerts
                            if !self.pending_alerts.contains(&alert_msg) {
                                self.pending_alerts.push(alert_msg.clone());
                                self.dialog = Dialog::Alert(alert_msg);
                                self.dialog_scroll_offset = 0;
                                
                                // Play sound if enabled
                                if self.config.alerts.sound_enabled {
                                    play_alert_sound();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Format a duration as hours:minutes:seconds
fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    format!("{}h {}m", hours, minutes)
}

/// Play an alert sound using rodio
fn play_alert_sound() {
    // Spawn sound in separate thread to avoid blocking UI
    std::thread::spawn(|| {
        if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
            // Generate a simple beep
            let source = rodio::source::SineWave::new(880.0)
                .take_duration(std::time::Duration::from_millis(200));
            let _ = stream_handle.play_raw(source.convert_samples());
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });
}
