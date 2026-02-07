use std::time::Duration;
use chrono::{DateTime, Utc};
use ratatui::widgets::TableState;

use crate::aws::{AwsClient, Ec2Instance, LambdaFunction};
use crate::config::AppConfig;
use crate::logger::LogManager;
use crate::settings::{Settings, SettingsField};

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
    Changelog,                // View Changelog
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

#[derive(Debug)]
pub enum AsyncNotification {
    SsoLoginSuccess(String, String), // Message, ProfileName
    SsoLoginFailed(String),
    ProfileActivated(crate::aws::AwsClient, String), // Client, ProfileName
    ProfileActivationFailed(String),
    Ec2Refreshed(Result<Vec<Ec2Instance>, String>),
    LambdaRefreshed(Result<Vec<LambdaFunction>, String>),
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
