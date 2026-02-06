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
use crate::logger::LogManager;
use crate::settings::{Settings, SettingsField};
use crate::ui;

/// Current screen/tab
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    #[default]
    Home,
    Ec2,
    Lambda,
    Logs,
    About,
}

/// Dialog/modal state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Dialog {
    None,
    Help,                     // Help popup
    Setup,                    // AWS setup instructions
    Settings,                 // Settings configuration
    SessionExpired,           // AWS session/token expired
    ConfirmTerminate(String), // instance_id
    ScheduleAutoStop(String), // instance_id  
    Alert(String),            // message
    ConfigureAws,             // AWS configuration/login instructions
}

/// Toast notification
#[derive(Debug, Clone)]
pub struct Toast {
    pub message: String,
    pub toast_type: ToastType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)] // Info may be used in future
pub enum ToastType {
    Success,
    Error,
    Info,
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
    
    // Auto-refresh
    pub last_refresh: Option<DateTime<Utc>>,
    pub auto_refresh_interval: Duration,
    pub boost_refresh_until_stable: bool, // Boost until all instances reach stable states
    
    // Toast notifications
    pub toasts: Vec<Toast>,
    
    // Window state
    pub window_size: (u16, u16),
    pub dialog_scroll_offset: u16,
    
    // Settings
    pub settings: Settings,
    pub settings_selected_field: SettingsField,
    pub settings_draft: Option<Settings>, // Draft while editing
    
    // Logging
    pub log_manager: LogManager,

    // Async Notifications
    pub async_tx: std::sync::mpsc::Sender<AsyncNotification>,
    pub async_rx: std::sync::mpsc::Receiver<AsyncNotification>,

    // AWS Profiles
    pub available_profiles: Vec<String>,
    pub selected_profile_index: usize,
    pub active_profile_name: Option<String>,
}

#[derive(Debug)]
pub enum AsyncNotification {
    SsoLoginSuccess(String, String), // Message, ProfileName
    SsoLoginFailed(String),
    ProfileActivated(crate::aws::AwsClient, String), // Client, ProfileName
    ProfileActivationFailed(String),
}


impl App {
    /// Create a new application instance
    pub async fn new() -> Result<Self> {
        let config = AppConfig::default();
        let aws_client = AwsClient::new(config.aws_region.as_deref()).await?;
        
        // Initialize logger first
        let mut log_manager = LogManager::new();
        log_manager.info("Application started".to_string());
        
        // Load settings from file
        let settings = match Settings::load() {
            Ok(s) => {
                log_manager.info("Settings loaded successfully".to_string());
                s
            }
            Err(e) => {
                log_manager.warning(format!("Failed to load settings, using defaults: {}", e));
                Settings::default()
            }
        };
        
        // Check if AWS credentials are configured by trying a simple operation
        let aws_configured = Self::check_aws_credentials().await;
        let initial_dialog = if aws_configured {
            Dialog::None
        } else {
            Dialog::Setup
        };
        
        if !aws_configured {
            log_manager.warning("AWS credentials not configured".to_string());
        } else {
            log_manager.info("AWS credentials detected".to_string());
        }
        
        let (async_tx, async_rx) = std::sync::mpsc::channel();
        
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
            last_refresh: None,
            auto_refresh_interval: settings.refresh_interval(),
            boost_refresh_until_stable: false,
            toasts: Vec::new(),
            scroll_offset: 0,
            window_size: (80, 24), // Default, will be updated
            dialog_scroll_offset: 0,
            settings,
            settings_selected_field: SettingsField::RefreshInterval,
            settings_draft: None,
            log_manager,
            async_tx,
            async_rx,
            available_profiles: crate::aws::list_aws_profiles().unwrap_or_default(),
            selected_profile_index: 0,
            active_profile_name: std::env::var("AWS_PROFILE").ok().or(Some("default".to_string())),
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

            // Check for async notifications (SSO login results)
            if let Err(e) = self.check_async_notifications().await {
                self.log_manager.error(format!("Notification error: {}", e));
            }

            // Check for alerts periodically
            self.check_alerts();
            
            // Auto-refresh if interval elapsed
            self.check_auto_refresh().await?;

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
                    4 => Screen::Logs,
                    _ => self.current_screen,
                };
                
                // Skip Logs screen if disabled in settings
                let new_screen = if new_screen == Screen::Logs && !self.settings.show_logs_panel {
                    self.current_screen
                } else {
                    new_screen
                };
                
                if new_screen != self.current_screen {
                    self.current_screen = new_screen;
                    self.scroll_offset = 0;
                    self.log_manager.info(format!("Navigated to {:?} screen", new_screen));
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
            
            AppEvent::OpenSettings => {
                self.open_settings_dialog();
            }
            
            // These are only used in settings dialog
            AppEvent::ModifySettingValue(_) | AppEvent::CancelSettings => {}
            
            AppEvent::None => {},
            AppEvent::ConfigureAws => {
                self.dialog = Dialog::ConfigureAws;
                self.dialog_scroll_offset = 0;
            }
            AppEvent::SsoLogin => {} // Only handled in dialogs
        }

        Ok(())
    }

    /// Handle events when dialog is open
    async fn handle_dialog_event(&mut self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::Quit => {
                if self.dialog == Dialog::Settings {
                    self.cancel_settings();
                } else {
                    self.dialog = Dialog::None;
                }
            }
            AppEvent::Up => {
                if self.dialog == Dialog::Settings {
                    self.navigate_settings_field(true);
                } else if matches!(self.dialog, Dialog::ConfigureAws | Dialog::SessionExpired) {
                    if self.selected_profile_index > 0 {
                        self.selected_profile_index -= 1;
                    }
                } else {
                    self.dialog_scroll_offset = self.dialog_scroll_offset.saturating_sub(1);
                }
            }
            AppEvent::Down => {
                if self.dialog == Dialog::Settings {
                    self.navigate_settings_field(false);
                } else if matches!(self.dialog, Dialog::ConfigureAws | Dialog::SessionExpired) {
                     if !self.available_profiles.is_empty() && self.selected_profile_index < self.available_profiles.len().saturating_sub(1) {
                        self.selected_profile_index += 1;
                    }
                } else {
                    // Calculate max scroll for current dialog based on window size
                    let (_, h) = self.window_size;
                    
                    let (percent_y, content_lines): (u16, u16) = match self.dialog {
                        Dialog::Setup => (70, 27),
                        Dialog::Help => (60, 27),
                        Dialog::Settings => (60, 15),
                        Dialog::SessionExpired => (60, 25),
                        Dialog::ConfirmTerminate(_) => (30, 12),
                        Dialog::ScheduleAutoStop(_) => (30, 12),
                        Dialog::Alert(_) => (25, 10),
                        Dialog::ConfigureAws => (50, 30), // Increased for profile list
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
            }
            AppEvent::Enter => {
                let current_dialog = self.dialog.clone();
                match current_dialog {
                    Dialog::ConfirmTerminate(id) => {
                        self.dialog = Dialog::None;
                        self.terminate_instance(&id).await?;
                    }
                    Dialog::ScheduleAutoStop(id) => {
                        self.dialog = Dialog::None;
                        self.schedule_auto_stop(&id, Duration::from_secs(3600))?;
                    }
                    Dialog::Settings => {
                        if self.settings_selected_field == SettingsField::TestSound {
                            self.trigger_test_alert();
                        } else {
                            self.save_settings();
                        }
                    }
                    Dialog::ConfigureAws | Dialog::SessionExpired => {
                         if !self.available_profiles.is_empty() {
                             let profile = self.available_profiles[self.selected_profile_index].clone();
                             self.activate_profile(&profile).await?;
                         }
                    }
                    Dialog::Alert(_) | Dialog::Help | Dialog::Setup => {
                        self.dialog = Dialog::None;
                    }
                    Dialog::None => {}
                }
            }
            AppEvent::ConfigureAws => {
                self.dialog = Dialog::ConfigureAws;
                self.dialog_scroll_offset = 0;
            }
            // Settings dialog specific events
            AppEvent::ModifySettingValue(delta) => {
                if self.dialog == Dialog::Settings {
                    self.modify_current_setting(delta);
                }
            }
            AppEvent::CancelSettings => {
                if self.dialog == Dialog::Settings {
                    self.cancel_settings();
                } else {
                    self.dialog = Dialog::None;
                }
            }
            AppEvent::SsoLogin => {
                if matches!(self.dialog, Dialog::SessionExpired | Dialog::ConfigureAws | Dialog::Setup) {
                    self.login_with_sso().await?;
                }
            }
            AppEvent::Refresh => {
                // Allow refreshing even when dialog is open (e.g. for SessionExpired retry)
                // If SessionExpired, attempting refresh might close it if successful
                self.refresh_data().await?;
                
                // If refresh success (no error), invalidating SessionExpired dialog happens in refresh_data?
                // refresh_data SETS SessionExpired if error. It doesn't UNSET it if success?
                // I need to check refresh_data logic. 
                // Line 453: if successful, it updates data. It doesn't modify self.dialog usually.
                // So I should close the dialog if refresh succeeds!
                
                // But refresh_data returns Result<()>.
                // It updates status_message.
                // If I'm in SessionExpired, and I press 'r', I want to close popup if fixed.
                // Let's modify logic:
                if self.dialog == Dialog::SessionExpired {
                     // If refresh works, we should close dialog.
                     // But refresh_data handles errors by setting Dialog::SessionExpired.
                     // So if we reset dialog to None BEFORE calling check, it might fail and re-open?
                     // Or check success.
                     
                     // Simpler: Just call refresh_data. If it fails, it re-opens/keeps open SessionExpired.
                     // If it succeeds, the dialog REMAINS OPEN with old state??
                     // Yes.
                     
                     // So I should explicitly close dialog if refresh succeeds?
                     // How to know if it succeeded?
                     // refresh_data doesn't return status.
                     
                     // Workaround: Reset dialog to None, then call refresh_data.
                     self.dialog = Dialog::None;
                     self.refresh_data().await?;
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
                    let new_idx = self.ec2_selected as i32 + delta;
                    self.ec2_selected = new_idx.clamp(0, (len - 1) as i32) as usize;
                }
            }
            Screen::Lambda => {
                let len = self.lambda_functions.len();
                if len > 0 {
                    let new_idx = self.lambda_selected as i32 + delta;
                    self.lambda_selected = new_idx.clamp(0, (len - 1) as i32) as usize;
                }
            }
            Screen::Home | Screen::About | Screen::Logs => {
                let (w, h) = self.window_size;
                // Estimate available height (minus headers, borders, footer)
                // Tabs(3) + Status(3) + Borders(2) = 8.
                let available_height = h.saturating_sub(8);
                
                let content_height: u16 = match self.current_screen {
                    // Home content height depends on width (wide vs narrow layout)
                    Screen::Home => if w >= 100 { 18 } else { 25 },
                    // About content height also depends on width (side-by-side vs stacked)
                    Screen::About => if w >= 100 { 30 } else { 58 },
                    // Logs screen - scroll through log entries
                    Screen::Logs => 50,
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
                        let count = instances.len();
                        self.ec2_instances = instances;
                        self.status_message = format!("Loaded {} EC2 instances", count);
                        self.log_manager.success(format!("Refreshed EC2: {} instances loaded", count));
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        self.status_message = format!("Error: {}", error_str);
                        self.log_manager.error(format!("Failed to load EC2 instances: {}", error_str));
                        
                        // Debug: log the full error for troubleshooting
                        self.log_manager.debug(format!("Raw error message for session check: {}", error_str));
                        
                        // Check for session expired errors
                        let is_expired = Self::is_session_expired_error(&error_str);
                        self.log_manager.debug(format!("Session expired check result: {}", is_expired));
                        
                        if is_expired {
                            self.dialog = Dialog::SessionExpired;
                            self.dialog_scroll_offset = 0;
                            self.log_manager.warning("AWS session token expired - credentials need refresh".to_string());
                        }
                    }
                }
            }
            Screen::Lambda => {
                match self.aws_client.list_lambda_functions().await {
                    Ok(functions) => {
                        let count = functions.len();
                        self.lambda_functions = functions;
                        self.status_message = format!("Loaded {} Lambda functions", count);
                        self.log_manager.success(format!("Refreshed Lambda: {} functions loaded", count));
                    }
                    Err(e) => {
                        let error_str = e.to_string();
                        self.status_message = format!("Error: {}", error_str);
                        self.log_manager.error(format!("Failed to load Lambda functions: {}", error_str));
                        
                        // Debug: log the full error for troubleshooting
                        self.log_manager.debug(format!("Raw error message for session check: {}", error_str));
                        
                        // Check for session expired errors
                        let is_expired = Self::is_session_expired_error(&error_str);
                        self.log_manager.debug(format!("Session expired check result: {}", is_expired));
                        
                        if is_expired {
                            self.dialog = Dialog::SessionExpired;
                            self.dialog_scroll_offset = 0;
                            self.log_manager.warning("AWS session token expired - credentials need refresh".to_string());
                        }
                    }
                }
            }
            Screen::About | Screen::Logs => {
                // Static screens, nothing to refresh
                self.status_message = "Nothing to refresh on this screen".to_string();
            }
        }

        self.is_loading = false;
        self.last_refresh = Some(Utc::now());
        Ok(())
    }
    
    /// Check if auto-refresh should trigger
    async fn check_auto_refresh(&mut self) -> Result<()> {
        // Skip auto-refresh on About screen or if dialog is open
        if self.current_screen == Screen::About || self.dialog != Dialog::None {
            return Ok(());
        }
        
        // Cleanup old toasts
        self.cleanup_old_toasts();
        
        let now = Utc::now();
        
        // Check if we should disable boost mode (all instances stable)
        if self.boost_refresh_until_stable {
            if self.all_instances_stable() {
                self.boost_refresh_until_stable = false;
            }
        }
        
        // Determine refresh interval (boost mode uses 5 seconds)
        let interval = if self.boost_refresh_until_stable {
            Duration::from_secs(5) // Fast refresh during state changes
        } else {
            self.auto_refresh_interval
        };
        
        let should_refresh = match self.last_refresh {
            Some(last) => {
                let elapsed = now.signed_duration_since(last);
                elapsed.num_seconds() as u64 >= interval.as_secs()
            }
            None => true, // First time, refresh immediately
        };
        
        if should_refresh {
            self.refresh_data().await?;
        }
        
        Ok(())
    }
    
    /// Get seconds until next refresh (for UI display)
    pub fn seconds_until_refresh(&self) -> Option<u64> {
        if self.current_screen == Screen::About || self.dialog != Dialog::None {
            return None;
        }
        
        let now = Utc::now();
        
        // Determine interval (boost mode uses 5 seconds)
        let interval = if self.boost_refresh_until_stable {
            Duration::from_secs(5)
        } else {
            self.auto_refresh_interval
        };
        
        self.last_refresh.map(|last| {
            let elapsed = now.signed_duration_since(last).num_seconds() as u64;
            interval.as_secs().saturating_sub(elapsed)
        })
    }
    
    /// Check if all instances are in stable states
    fn all_instances_stable(&self) -> bool {
        self.ec2_instances.iter().all(|instance| {
            matches!(instance.state.as_str(), "running" | "stopped" | "terminated")
        })
    }
    
    /// Activate boost refresh mode (until instances stabilize)
    fn activate_boost_refresh(&mut self) {
        self.boost_refresh_until_stable = true;
    }
    
    /// Add a toast notification
    pub fn add_toast(&mut self, message: String, toast_type: ToastType) {
        self.toasts.push(Toast {
            message,
            toast_type,
            created_at: Utc::now(),
        });
    }
    
    /// Activate a specific AWS profile
    pub async fn activate_profile(&mut self, profile_name: &str) -> Result<()> {
        self.status_message = format!("Switching to profile: {}...", profile_name);
        self.add_toast(format!("🔄 Switching to profile '{}'...", profile_name), ToastType::Info);
        self.is_loading = true; // Enable loading state
        
        // Critical: Set the AWS_PROFILE env var so the new client picks it up (for THIS thread/process)
        std::env::set_var("AWS_PROFILE", profile_name);
        self.log_manager.info(format!("Set AWS_PROFILE={} and re-initializing client", profile_name));

        let tx = self.async_tx.clone();
        let region = self.config.aws_region.clone();
        let profile_name_owned = profile_name.to_string();

        // Spawn background task to init client
        tokio::spawn(async move {
            match crate::aws::AwsClient::new(region.as_deref()).await {
                Ok(client) => {
                    let _ = tx.send(AsyncNotification::ProfileActivated(client, profile_name_owned));
                },
                Err(e) => {
                    let _ = tx.send(AsyncNotification::ProfileActivationFailed(e.to_string()));
                }
            }
        });

        Ok(())
    }
    
    /// Check for async notifications from threads
    pub async fn check_async_notifications(&mut self) -> Result<()> {
        // Try to receive all pending messages
        while let Ok(notification) = self.async_rx.try_recv() {
            match notification {
                AsyncNotification::SsoLoginSuccess(msg, profile) => {
                     self.add_toast("✅ Login successful! Activating profile...".to_string(), ToastType::Success);
                     self.log_manager.success(format!("{}: {}", msg, profile));
                     
                     if let Err(e) = self.activate_profile(&profile).await {
                         self.log_manager.error(format!("Failed to activate profile after login: {}", e));
                     }
                }
                AsyncNotification::SsoLoginFailed(err) => {
                     // Check for specific SSO config error to show cleaner message
                     if err.contains("Missing the following required SSO configuration") {
                         self.add_toast("❌ SSO Config Missing. Run 'aws configure sso'".to_string(), ToastType::Error);
                         self.log_manager.error(format!("SSO Config Error: {}", err));
                         self.status_message = "SSO Configuration Missing!".to_string();
                     } else {
                         self.add_toast(format!("❌ Login Failed: {}", err), ToastType::Error);
                         self.log_manager.error(format!("Login failed: {}", err));
                     }
                }
                AsyncNotification::ProfileActivated(client, profile_name) => {
                    self.aws_client = client;
                    self.aws_configured = true;
                    self.active_profile_name = Some(profile_name.clone());
                    self.is_loading = false;
                    self.dialog = Dialog::None; // Close any open dialogs (Configure/SessionExpired)
                    self.add_toast(format!("✅ Active Profile: {}", profile_name), ToastType::Success);
                    if let Err(e) = self.refresh_data().await {
                        self.log_manager.error(format!("Failed to refresh data after profile switch: {}", e));
                    }
                }
                AsyncNotification::ProfileActivationFailed(err) => {
                    self.is_loading = false;
                    self.log_manager.error(format!("Failed to switch profile: {}", err));
                    self.add_toast("Failed to switch profile".to_string(), ToastType::Error);
                }
            }
        }
        Ok(())
    }

    /// Remove toasts older than 5 seconds
    fn cleanup_old_toasts(&mut self) {
        let now = Utc::now();
        self.toasts.retain(|toast| {
            now.signed_duration_since(toast.created_at).num_seconds() < 5
        });
    }

    /// Start the selected EC2 instance
    async fn start_selected_instance(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            let id = instance.id.clone();
            let name = instance.name.clone();
            self.status_message = format!("Starting {}...", id);
            
            match self.aws_client.start_instance(&id).await {
                Ok(_) => {
                    self.status_message = format!("Started {}", id);
                    self.add_toast(format!("✓ Started: {}", name), ToastType::Success);
                    self.log_manager.success(format!("Started EC2 instance: {} ({})", name, id));
                    self.activate_boost_refresh();
                    self.refresh_data().await?;
                }
                Err(e) => {
                    self.status_message = format!("Failed to start: {}", e);
                    self.add_toast(format!("✗ Failed to start: {}", name), ToastType::Error);
                    self.log_manager.error(format!("Failed to start {}: {}", name, e));
                }
            }
        }
        Ok(())
    }

    /// Stop the selected EC2 instance
    async fn stop_selected_instance(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            let id = instance.id.clone();
            let name = instance.name.clone();
            self.status_message = format!("Stopping {}...", id);
            
            match self.aws_client.stop_instance(&id).await {
                Ok(_) => {
                    self.status_message = format!("Stopped {}", id);
                    self.add_toast(format!("✓ Stopped: {}", name), ToastType::Success);
                    self.log_manager.success(format!("Stopped EC2 instance: {} ({})", name, id));
                    self.activate_boost_refresh();
                    self.refresh_data().await?;
                }
                Err(e) => {
                    self.status_message = format!("Failed to stop: {}", e);
                    self.add_toast(format!("✗ Failed to stop: {}", name), ToastType::Error);
                    self.log_manager.error(format!("Failed to stop {}: {}", name, e));
                }
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
        // Find instance name
        let instance_name = self.ec2_instances.iter()
            .find(|i| i.id == instance_id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| instance_id.to_string());
            
        self.status_message = format!("Terminating {}...", instance_id);
        
        match self.aws_client.terminate_instance(instance_id).await {
            Ok(_) => {
                self.status_message = format!("Terminated {}", instance_id);
                self.add_toast(format!("✓ Terminated: {}", instance_name), ToastType::Success);
                self.log_manager.success(format!("Terminated EC2 instance: {} ({})", instance_name, instance_id));
                self.activate_boost_refresh();
                self.refresh_data().await?;
            }
            Err(e) => {
                self.status_message = format!("Failed to terminate: {}", e);
                self.add_toast(format!("✗ Failed to terminate: {}", instance_name), ToastType::Error);
                self.log_manager.error(format!("Failed to terminate {}: {}", instance_name, e));
            }
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
        
        // Find instance name for logging
        let instance_name = self.ec2_instances.iter()
            .find(|i| i.id == instance_id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| instance_id.to_string());
        
        // Remove existing schedule for this instance
        self.auto_stop_schedules.retain(|(id, _)| id != instance_id);
        
        // Add new schedule
        self.auto_stop_schedules.push((instance_id.to_string(), stop_time));
        self.status_message = format!("Scheduled auto-stop for {} at {}", instance_id, stop_time.format("%H:%M:%S"));
        self.add_toast(format!("⏰ Scheduled: {}", instance_name), ToastType::Success);
        self.log_manager.success(format!("Scheduled auto-stop for {} ({}) at {}", instance_name, instance_id, stop_time.format("%H:%M:%S")));
        
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
            Screen::About | Screen::Logs => {
                // These screens have no interactive elements
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
    
    /// Open the settings dialog
    fn open_settings_dialog(&mut self) {
        // Create a draft copy of settings for editing
        self.settings_draft = Some(self.settings.clone());
        self.settings_selected_field = SettingsField::RefreshInterval;
        self.dialog = Dialog::Settings;
        self.dialog_scroll_offset = 0;
        self.log_manager.info("Opened settings dialog".to_string());
    }
    
    /// Save settings and close dialog
    fn save_settings(&mut self) {
        if let Some(draft) = self.settings_draft.take() {
            self.settings = draft;
            self.auto_refresh_interval = self.settings.refresh_interval();
            
            // Save to file
            if let Err(e) = self.settings.save() {
                self.add_toast(format!("Failed to save settings: {}", e), ToastType::Error);
                self.log_manager.error(format!("Failed to save settings: {}", e));
            } else {
                self.add_toast("Settings saved".to_string(), ToastType::Success);
                self.log_manager.success("Settings saved".to_string());
            }
        }
        self.dialog = Dialog::None;
    }
    
    /// Cancel settings and close dialog
    fn cancel_settings(&mut self) {
        self.settings_draft = None;
        self.dialog = Dialog::None;
        self.log_manager.info("Settings dialog cancelled".to_string());
    }
    
    /// Modify the currently selected setting
    fn modify_current_setting(&mut self, delta: i32) {
        if let Some(ref mut draft) = self.settings_draft {
            let forward = delta > 0;
            match self.settings_selected_field {
                SettingsField::RefreshInterval => draft.cycle_refresh_interval(forward),
                SettingsField::ShowLogsPanel => draft.toggle_logs_panel(),
                SettingsField::LogLevel => draft.cycle_log_level(forward),
                SettingsField::AlertThreshold => draft.cycle_alert_threshold(forward),
                SettingsField::SoundEnabled => draft.toggle_sound(),
                SettingsField::TestSound => {} // Action only, no value modification
            }
        }
    }
    
    /// Navigate settings fields (used in settings dialog)
    pub fn navigate_settings_field(&mut self, up: bool) {
        self.settings_selected_field = if up {
            self.settings_selected_field.prev()
        } else {
            self.settings_selected_field.next()
        };
    }
    
    /// Check if an error message indicates an expired session/token
    fn is_session_expired_error(error_msg: &str) -> bool {
        let error_lower = error_msg.to_lowercase();
        error_lower.contains("expiredtoken") ||
        error_lower.contains("expired token") ||
        error_lower.contains("token is expired") ||
        error_lower.contains("security token") ||
        error_lower.contains("invalidtoken") ||
        error_lower.contains("invalid token") ||
        error_lower.contains("credentials have expired") ||
        error_lower.contains("the security token included in the request is expired") ||
        error_lower.contains("requestexpired") ||
        error_lower.contains("request has expired") ||
        error_lower.contains("authfailure")
    }
}

/// Format a duration as hours:minutes:seconds
fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    format!("{}h {}m", hours, minutes)
}

impl App {
    /// Trigger a test alert with sound and notification
    fn trigger_test_alert(&mut self) {
        play_alert_sound();
        self.add_toast("🔔 Test Alert: System Sound Working".to_string(), ToastType::Info);
        self.log_manager.info("Triggered test alert sound".to_string());
    }

    /// Trigger SSO login via AWS CLI
    pub async fn login_with_sso(&mut self) -> Result<()> {
        self.status_message = "Initiating AWS SSO Login...".to_string();
        self.add_toast("🔑 Starting AWS SSO login... check browser".to_string(), ToastType::Info);
        
        let tx = self.async_tx.clone();
        
        let profile = if !self.available_profiles.is_empty() {
             Some(self.available_profiles[self.selected_profile_index].clone())
        } else {
             None
        };

        // Spawn thread to run command and capture output
        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("aws");
            cmd.arg("sso").arg("login");
            
            if let Some(ref p) = profile {
                cmd.arg("--profile").arg(p);
            }
            
            match cmd.output() // Use output() to capture stdout/stderr
            {
                Ok(output) => {
                    if output.status.success() {
                        let profile_name = profile.clone().unwrap_or_else(|| "default".to_string());
                        let _ = tx.send(AsyncNotification::SsoLoginSuccess("Login successful".to_string(), profile_name));
                    } else {
                        // Capture stderr
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                        // Also try stdout if stderr is empty, or combine?
                        let err_msg = if stderr.trim().is_empty() {
                            String::from_utf8_lossy(&output.stdout).to_string()
                        } else {
                            stderr
                        };
                        let _ = tx.send(AsyncNotification::SsoLoginFailed(err_msg));
                    }
                }
                Err(e) => {
                     let _ = tx.send(AsyncNotification::SsoLoginFailed(e.to_string()));
                }
            }
        });
        
        self.log_manager.info("Spawned 'aws sso login' thread".to_string());
        Ok(())
    }
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
