use std::time::Duration;
use anyhow::Result;
use chrono::Utc;
use rodio::Source;
use crate::app::state::{App, AsyncNotification, Dialog, Screen, Toast, ToastType};
use crate::settings::SettingsField;

// Helper function to play sound
fn play_alert_sound() {
    std::thread::spawn(|| {
        if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
            let source = rodio::source::SineWave::new(880.0)
                .take_duration(std::time::Duration::from_millis(200));
            let _ = stream_handle.play_raw(source.convert_samples());
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });
}

// Helper to format duration
fn format_duration(duration: chrono::Duration) -> String {
    let hours = duration.num_hours();
    let minutes = duration.num_minutes() % 60;
    format!("{}h {}m", hours, minutes)
}

impl App {
    // --- Toast & Notification Methods ---

    /// Add a toast notification
    pub fn add_toast(&mut self, message: String, toast_type: ToastType) {
        self.toasts.push(Toast {
            message,
            toast_type,
            created_at: Utc::now(),
        });
    }

    /// Remove toasts older than 5 seconds
    pub fn cleanup_old_toasts(&mut self) {
        let now = Utc::now();
        self.toasts.retain(|toast| {
            now.signed_duration_since(toast.created_at).num_seconds() < 5
        });
    }

    // --- Refresh & Data Loading Methods ---

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
                        
                        if Self::is_session_expired_error(&error_str) {
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
                        
                        if Self::is_session_expired_error(&error_str) {
                            self.dialog = Dialog::SessionExpired;
                            self.dialog_scroll_offset = 0;
                            self.log_manager.warning("AWS session token expired - credentials need refresh".to_string());
                        }
                    }
                }
            }
            Screen::About | Screen::Logs => {
                self.status_message = "Nothing to refresh on this screen".to_string();
            }
        }

        self.is_loading = false;
        self.last_refresh = Some(Utc::now());
        Ok(())
    }
    
    /// Check if auto-refresh should trigger
    pub async fn check_auto_refresh(&mut self) -> Result<()> {
        if self.current_screen == Screen::About || self.dialog != Dialog::None {
            return Ok(());
        }
        
        self.cleanup_old_toasts();
        
        let now = Utc::now();
        
        if self.boost_refresh_until_stable && self.all_instances_stable() {
            self.boost_refresh_until_stable = false;
        }
        
        let interval = if self.boost_refresh_until_stable {
            Duration::from_secs(5)
        } else {
            self.auto_refresh_interval
        };
        
        let should_refresh = match self.last_refresh {
            Some(last) => {
                let elapsed = now.signed_duration_since(last);
                elapsed.num_seconds() as u64 >= interval.as_secs()
            }
            None => true,
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
    pub fn activate_boost_refresh(&mut self) {
        self.boost_refresh_until_stable = true;
    }

    // --- AWS & Instance Actions ---

    /// Start the selected EC2 instance
    pub async fn start_selected_instance(&mut self) -> Result<()> {
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
    pub async fn stop_selected_instance(&mut self) -> Result<()> {
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
    pub fn confirm_terminate_instance(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            self.dialog = Dialog::ConfirmTerminate(instance.id.clone());
            self.dialog_scroll_offset = 0;
        }
        Ok(())
    }

    /// Terminate an EC2 instance
    pub async fn terminate_instance(&mut self, instance_id: &str) -> Result<()> {
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
    pub fn open_schedule_dialog(&mut self) -> Result<()> {
        if let Some(instance) = self.ec2_instances.get(self.ec2_selected) {
            self.dialog = Dialog::ScheduleAutoStop(instance.id.clone());
            self.dialog_scroll_offset = 0;
        }
        Ok(())
    }

    /// Schedule auto-stop for an instance
    pub fn schedule_auto_stop(&mut self, instance_id: &str, duration: Duration) -> Result<()> {
        let stop_time = Utc::now() + chrono::Duration::from_std(duration)?;
        let instance_name = self.ec2_instances.iter()
            .find(|i| i.id == instance_id)
            .map(|i| i.name.clone())
            .unwrap_or_else(|| instance_id.to_string());
        
        self.auto_stop_schedules.retain(|(id, _)| id != instance_id);
        self.auto_stop_schedules.push((instance_id.to_string(), stop_time));
        
        self.status_message = format!("Scheduled auto-stop for {} at {}", instance_id, stop_time.format("%H:%M:%S"));
        self.add_toast(format!("⏰ Scheduled: {}", instance_name), ToastType::Success);
        self.log_manager.success(format!("Scheduled auto-stop for {} ({}) at {}", instance_name, instance_id, stop_time.format("%H:%M:%S")));
        Ok(())
    }

    /// Check for alerts
    pub fn check_alerts(&mut self) {
        let now = Utc::now();
        if let Some(last_check) = self.last_alert_check {
            if now.signed_duration_since(last_check).num_seconds() < 30 {
                return;
            }
        }
        self.last_alert_check = Some(now);

        let threshold = chrono::Duration::from_std(self.config.alerts.alert_threshold)
            .unwrap_or(chrono::Duration::hours(1));

        for instance in &self.ec2_instances {
            if instance.state == "running" {
                let has_schedule = self.auto_stop_schedules.iter().any(|(id, _)| *id == instance.id);
                if !has_schedule {
                    if let Some(launch_time) = instance.launch_time {
                        let running_duration = now.signed_duration_since(launch_time);
                        if running_duration > threshold {
                            let alert_msg = format!(
                                "⚠️ Instance {} ({}) running for {} without auto-stop!",
                                instance.name, instance.id, format_duration(running_duration)
                            );
                            if !self.pending_alerts.contains(&alert_msg) {
                                self.pending_alerts.push(alert_msg.clone());
                                self.dialog = Dialog::Alert(alert_msg);
                                self.dialog_scroll_offset = 0;
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
    
    // --- Settings Methods ---

    pub fn open_settings_dialog(&mut self) {
        self.settings_draft = Some(self.settings.clone());
        self.settings_selected_field = SettingsField::RefreshInterval;
        self.dialog = Dialog::Settings;
        self.dialog_scroll_offset = 0;
        self.log_manager.info("Opened settings dialog".to_string());
    }
    
    pub fn save_settings(&mut self) {
        if let Some(draft) = self.settings_draft.take() {
            self.settings = draft;
            self.auto_refresh_interval = self.settings.refresh_interval();
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
    
    pub fn cancel_settings(&mut self) {
        self.settings_draft = None;
        self.dialog = Dialog::None;
        self.log_manager.info("Settings dialog cancelled".to_string());
    }
    
    pub fn modify_current_setting(&mut self, delta: i32) {
        if let Some(ref mut draft) = self.settings_draft {
            let forward = delta > 0;
            match self.settings_selected_field {
                SettingsField::RefreshInterval => draft.cycle_refresh_interval(forward),
                SettingsField::ShowLogsPanel => draft.toggle_logs_panel(),
                SettingsField::LogLevel => draft.cycle_log_level(forward),
                SettingsField::AlertThreshold => draft.cycle_alert_threshold(forward),
                SettingsField::SoundEnabled => draft.toggle_sound(),
                SettingsField::TestSound => {} 
            }
        }
    }
    
    pub fn navigate_settings_field(&mut self, up: bool) {
        self.settings_selected_field = if up {
            self.settings_selected_field.prev()
        } else {
            self.settings_selected_field.next()
        };
    }

    /// Trigger a test alert
    pub fn trigger_test_alert(&mut self) {
        play_alert_sound();
        self.add_toast("🔔 Test Alert: System Sound Working".to_string(), ToastType::Info);
        self.log_manager.info("Triggered test alert sound".to_string());
    }

    // --- Auth & Profile Methods ---

    pub async fn login_with_sso(&mut self) -> Result<()> {
        self.status_message = "Initiating AWS SSO Login...".to_string();
        self.add_toast("🔑 Starting AWS SSO login... check browser".to_string(), ToastType::Info);
        
        let tx = self.async_tx.clone();
        let profile = if !self.available_profiles.is_empty() {
             Some(self.available_profiles[self.selected_profile_index].clone())
        } else {
             None
        };

        std::thread::spawn(move || {
            let mut cmd = std::process::Command::new("aws");
            cmd.arg("sso").arg("login");
            if let Some(ref p) = profile {
                cmd.arg("--profile").arg(p);
            }
            match cmd.output() {
                Ok(output) => {
                    if output.status.success() {
                        let profile_name = profile.clone().unwrap_or_else(|| "default".to_string());
                        let _ = tx.send(AsyncNotification::SsoLoginSuccess("Login successful".to_string(), profile_name));
                    } else {
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
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

    pub async fn activate_profile(&mut self, profile_name: &str) -> Result<()> {
        self.status_message = format!("Switching to profile: {}...", profile_name);
        self.add_toast(format!("🔄 Switching to profile '{}'...", profile_name), ToastType::Info);
        self.is_loading = true;
        
        std::env::set_var("AWS_PROFILE", profile_name);
        self.log_manager.info(format!("Set AWS_PROFILE={} and re-initializing client", profile_name));

        let tx = self.async_tx.clone();
        let region = self.config.aws_region.clone();
        let profile_name_owned = profile_name.to_string();

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

    pub async fn check_async_notifications(&mut self) -> Result<()> {
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
                    self.dialog = Dialog::None;
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

    // Helper for error checking
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
