//! Logging system for tracking user actions and application events

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Maximum number of log entries to keep in memory
const MAX_LOG_ENTRIES: usize = 1000;

/// Log entry level/severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogLevel {
    Debug,    // Verbose debugging information
    Info,     // General information
    Success,  // Successful operations
    Warning,  // Warnings that don't prevent operation
    Error,    // Errors that affect functionality
}

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: DateTime<Utc>,
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    /// Create a new log entry
    pub fn new(level: LogLevel, message: String) -> Self {
        Self {
            timestamp: Utc::now(),
            level,
            message,
        }
    }
}

/// Log manager - collects and manages log entries
pub struct LogManager {
    entries: Vec<LogEntry>,
    scroll_offset: usize,
}

impl Default for LogManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LogManager {
    /// Create a new log manager
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            scroll_offset: 0,
        }
    }
    
    /// Add a log entry
    pub fn log(&mut self, level: LogLevel, message: String) {
        self.entries.push(LogEntry::new(level, message));
        
        // Keep only the most recent entries
        if self.entries.len() > MAX_LOG_ENTRIES {
            self.entries.drain(0..(self.entries.len() - MAX_LOG_ENTRIES));
        }
    }
    
    /// Convenience method for debug/verbose logs
    #[allow(dead_code)]
    pub fn debug(&mut self, message: String) {
        self.log(LogLevel::Debug, message);
    }
    
    /// Convenience method for info logs
    pub fn info(&mut self, message: String) {
        self.log(LogLevel::Info, message);
    }
    
    /// Convenience method for success logs
    pub fn success(&mut self, message: String) {
        self.log(LogLevel::Success, message);
    }
    
    /// Convenience method for warning logs
    pub fn warning(&mut self, message: String) {
        self.log(LogLevel::Warning, message);
    }
    
    /// Convenience method for error logs
    pub fn error(&mut self, message: String) {
        self.log(LogLevel::Error, message);
    }
    
    /// Get all log entries (most recent last)
    pub fn entries(&self) -> &[LogEntry] {
        &self.entries
    }
    
    /// Get scroll offset for UI
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }
    
    /// Scroll up in logs
    #[allow(dead_code)]
    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }
    
    /// Scroll down in logs
    #[allow(dead_code)]
    pub fn scroll_down(&mut self, visible_height: usize) {
        let max_scroll = self.entries.len().saturating_sub(visible_height);
        if self.scroll_offset < max_scroll {
            self.scroll_offset += 1;
        }
    }
    
    /// Reset scroll to bottom (showing most recent)
    #[allow(dead_code)]
    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }
}
