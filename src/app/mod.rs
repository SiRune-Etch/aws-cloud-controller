//! Application state and core logic

pub mod actions;
pub mod handlers;
pub mod state;

use std::time::Duration;
use anyhow::Result;
use ratatui::backend::Backend;
use ratatui::Terminal;
use ratatui::widgets::TableState;

use crate::aws::AwsClient;
use crate::config::AppConfig;
use crate::event::poll_event;
use crate::logger::LogManager;
use crate::settings::{Settings, SettingsField};
use crate::ui;

// Re-export core types for external usage (like main.rs)
pub use state::{App, Screen, Dialog, ToastType};

impl App {
    /// Create a new application instance
    pub async fn new() -> Result<Self> {
        let config = AppConfig::default();
        
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
        
        // Load available profiles
        let available_profiles = crate::aws::list_aws_profiles().unwrap_or_default();
        
        // Set default profile if configured and available
        if let Some(default_profile) = &settings.default_profile {
            if available_profiles.contains(default_profile) {
                std::env::set_var("AWS_PROFILE", default_profile);
                log_manager.info(format!("Using default profile: {}", default_profile));
            } else {
                log_manager.warning(format!("Default profile '{}' not found in available profiles", default_profile));
            }
        }
        
        // Initialize AWS client (now that AWS_PROFILE is set)
        let aws_client = AwsClient::new(config.aws_region.as_deref()).await?;
        
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
            available_profiles: available_profiles.clone(),
            selected_profile_index: if let Ok(current) = std::env::var("AWS_PROFILE") {
                available_profiles.iter().position(|p| p == &current).unwrap_or(0)
            } else {
                0
            },
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
}
