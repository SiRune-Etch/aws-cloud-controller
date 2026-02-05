//! Application configuration

use std::time::Duration;

/// Alert configuration for long-running instances
pub struct AlertConfig {
    /// Duration after which to alert if instance is running without auto-stop
    pub alert_threshold: Duration,
    /// Whether sound alerts are enabled
    pub sound_enabled: bool,
    /// Slack webhook URL (for future implementation)
    #[allow(dead_code)] // Planned feature
    pub slack_webhook_url: Option<String>,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            alert_threshold: Duration::from_secs(3600), // 1 hour
            sound_enabled: true,
            slack_webhook_url: None,
        }
    }
}

/// Application-wide configuration
pub struct AppConfig {
    /// AWS region override (None = use default chain)
    pub aws_region: Option<String>,
    /// Alert settings
    pub alerts: AlertConfig,
    /// Tick rate for UI refresh in milliseconds
    pub tick_rate_ms: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            aws_region: None,
            alerts: AlertConfig::default(),
            tick_rate_ms: 250,
        }
    }
}
