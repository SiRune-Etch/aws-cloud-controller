//! Settings management with persistent storage

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

use crate::logger::LogLevel;

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    /// Auto-refresh interval in seconds
    pub refresh_interval_secs: u64,
    /// Whether to show logs panel/tab
    pub show_logs_panel: bool,
    /// Minimum log level to display
    #[serde(default = "default_log_level")]
    pub log_level: LogLevel,
    /// Alert threshold in seconds for long-running instances
    pub alert_threshold_secs: u64,
    /// Whether sound alerts are enabled
    pub sound_enabled: bool,
}

fn default_log_level() -> LogLevel {
    LogLevel::Info
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            refresh_interval_secs: 60,     // 60 seconds
            show_logs_panel: false,        // Hidden by default, enable via settings
            log_level: LogLevel::Info,     // Show Info and above by default
            alert_threshold_secs: 3600,    // 1 hour
            sound_enabled: true,
        }
    }
}

impl Settings {
    /// Get the config directory path (platform-specific)
    pub fn get_config_dir() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Failed to get config directory")?
            .join("aws-cloud-controller");
        
        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .context("Failed to create config directory")?;
        }
        
        Ok(config_dir)
    }
    
    /// Get the settings file path
    fn get_settings_path() -> Result<PathBuf> {
        Ok(Self::get_config_dir()?.join("settings.json"))
    }
    
    /// Load settings from file, or return defaults if file doesn't exist
    pub fn load() -> Result<Self> {
        let path = Self::get_settings_path()?;
        
        if !path.exists() {
            // Create default settings file
            let default_settings = Self::default();
            default_settings.save()?;
            return Ok(default_settings);
        }
        
        let contents = fs::read_to_string(&path)
            .context("Failed to read settings file")?;
        
        let settings: Settings = serde_json::from_str(&contents)
            .context("Failed to parse settings file")?;
        
        Ok(settings)
    }
    
    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        let path = Self::get_settings_path()?;
        
        let contents = serde_json::to_string_pretty(self)
            .context("Failed to serialize settings")?;
        
        fs::write(&path, contents)
            .context("Failed to write settings file")?;
        
        Ok(())
    }
    
    /// Get refresh interval as Duration
    pub fn refresh_interval(&self) -> Duration {
        Duration::from_secs(self.refresh_interval_secs)
    }
    
    /// Get alert threshold as Duration 
    #[allow(dead_code)]
    pub fn alert_threshold(&self) -> Duration {
        Duration::from_secs(self.alert_threshold_secs)
    }
    
    /// Cycle refresh interval to next value
    pub fn cycle_refresh_interval(&mut self, forward: bool) {
        const INTERVALS: &[u64] = &[15, 30, 60, 120, 300]; // 15s, 30s, 1m, 2m, 5m
        
        let current_idx = INTERVALS.iter()
            .position(|&x| x == self.refresh_interval_secs)
            .unwrap_or(2); // Default to 60s if not found
        
        let new_idx = if forward {
            (current_idx + 1) % INTERVALS.len()
        } else {
            if current_idx == 0 {
                INTERVALS.len() - 1
            } else {
                current_idx - 1
            }
        };
        
        self.refresh_interval_secs = INTERVALS[new_idx];
    }
    
    /// Cycle alert threshold to next value
    pub fn cycle_alert_threshold(&mut self, forward: bool) {
        const THRESHOLDS: &[u64] = &[1800, 3600, 7200, 14400, 28800]; // 30m, 1h, 2h, 4h, 8h
        
        let current_idx = THRESHOLDS.iter()
            .position(|&x| x == self.alert_threshold_secs)
            .unwrap_or(1); // Default to 1h if not found
        
        let new_idx = if forward {
            (current_idx + 1) % THRESHOLDS.len()
        } else {
            if current_idx == 0 {
                THRESHOLDS.len() - 1
            } else {
                current_idx - 1
            }
        };
        
        self.alert_threshold_secs = THRESHOLDS[new_idx];
    }
    
    /// Toggle show logs panel
    pub fn toggle_logs_panel(&mut self) {
        self.show_logs_panel = !self.show_logs_panel;
    }
    
    /// Toggle sound alerts
    pub fn toggle_sound(&mut self) {
        self.sound_enabled = !self.sound_enabled;
    }
    
    /// Format refresh interval for display
    pub fn format_refresh_interval(&self) -> String {
        if self.refresh_interval_secs < 60 {
            format!("{}s", self.refresh_interval_secs)
        } else if self.refresh_interval_secs < 3600 {
            format!("{}m", self.refresh_interval_secs / 60)
        } else {
            format!("{}h", self.refresh_interval_secs / 3600)
        }
    }
    
    /// Format alert threshold for display
    pub fn format_alert_threshold(&self) -> String {
        if self.alert_threshold_secs < 60 {
            format!("{}s", self.alert_threshold_secs)
        } else if self.alert_threshold_secs < 3600 {
            format!("{}m", self.alert_threshold_secs / 60)
        } else {
            format!("{}h", self.alert_threshold_secs / 3600)
        }
    }
    
    /// Cycle log level to next value
    pub fn cycle_log_level(&mut self, forward: bool) {
        // Debug -> Info -> Warning -> Error
        self.log_level = if forward {
            match self.log_level {
                LogLevel::Debug => LogLevel::Info,
                LogLevel::Info => LogLevel::Warning,
                LogLevel::Warning => LogLevel::Error,
                LogLevel::Error => LogLevel::Debug,
                LogLevel::Success => LogLevel::Info, // Normalize to Info
            }
        } else {
            match self.log_level {
                LogLevel::Debug => LogLevel::Error,
                LogLevel::Info => LogLevel::Debug,
                LogLevel::Warning => LogLevel::Info,
                LogLevel::Error => LogLevel::Warning,
                LogLevel::Success => LogLevel::Info, // Normalize to Info
            }
        };
    }
    
    /// Format log level for display
    pub fn format_log_level(&self) -> String {
        match self.log_level {
            LogLevel::Debug => "Debug (All)".to_string(),
            LogLevel::Info => "Info".to_string(),
            LogLevel::Warning => "Warning".to_string(),
            LogLevel::Error => "Error Only".to_string(),
            LogLevel::Success => "Info".to_string(),
        }
    }
    
    /// Check if a log level should be displayed based on current setting
    pub fn should_show_log(&self, level: LogLevel) -> bool {
        match self.log_level {
            LogLevel::Debug => true, // Show all
            LogLevel::Info => !matches!(level, LogLevel::Debug),
            LogLevel::Warning => matches!(level, LogLevel::Warning | LogLevel::Error),
            LogLevel::Error => matches!(level, LogLevel::Error),
            LogLevel::Success => !matches!(level, LogLevel::Debug),
        }
    }
}

/// Which field in the settings dialog is currently selected
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsField {
    RefreshInterval,
    ShowLogsPanel,
    LogLevel,
    AlertThreshold,
    SoundEnabled,
    TestSound,
}

impl SettingsField {
    /// Get the next field
    pub fn next(&self) -> Self {
        match self {
            Self::RefreshInterval => Self::ShowLogsPanel,
            Self::ShowLogsPanel => Self::LogLevel,
            Self::LogLevel => Self::AlertThreshold,
            Self::AlertThreshold => Self::SoundEnabled,
            Self::SoundEnabled => Self::TestSound,
            Self::TestSound => Self::RefreshInterval,
        }
    }
    
    /// Get the previous field
    pub fn prev(&self) -> Self {
        match self {
            Self::RefreshInterval => Self::TestSound,
            Self::ShowLogsPanel => Self::RefreshInterval,
            Self::LogLevel => Self::ShowLogsPanel,
            Self::AlertThreshold => Self::LogLevel,
            Self::SoundEnabled => Self::AlertThreshold,
            Self::TestSound => Self::SoundEnabled,
        }
    }
}
