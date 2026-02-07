use std::time::Duration;
use anyhow::Result;
use crate::app::state::{App, Dialog, Screen};
use crate::event::AppEvent;
use crate::settings::SettingsField;

impl App {
    /// Handle application events
    pub async fn handle_event(&mut self, event: AppEvent) -> Result<()> {
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
            AppEvent::ShowChangelog => {
                if self.current_screen == Screen::About {
                    self.dialog = Dialog::Changelog;
                    self.dialog_scroll_offset = 0;
                }
            }
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
                    if self.settings_selected_field != SettingsField::RefreshInterval {
                        self.navigate_settings_field(true);
                        self.ensure_dialog_selection_visible();
                    } else {
                        self.dialog_scroll_offset = self.dialog_scroll_offset.saturating_sub(1);
                    }
                } else if matches!(self.dialog, Dialog::ConfigureAws | Dialog::SessionExpired) {
                    if self.selected_profile_index > 0 {
                        self.selected_profile_index -= 1;
                        self.ensure_dialog_selection_visible();
                    } else {
                        self.dialog_scroll_offset = self.dialog_scroll_offset.saturating_sub(1);
                    }
                } else {
                    self.dialog_scroll_offset = self.dialog_scroll_offset.saturating_sub(1);
                }
            }
            AppEvent::Down => {
                // Calculate max scroll for current dialog based on window size to handle scrolling past selection
                let (_, h) = self.window_size;
                
                let (percent_y, content_lines): (u16, u16) = match self.dialog {
                    Dialog::Setup => (70, 27),
                    Dialog::Help => (60, 27),
                    Dialog::Settings => (60, 15),
                    Dialog::SessionExpired => (60, 25),
                    Dialog::ConfirmTerminate(_) => (30, 12),
                    Dialog::ScheduleAutoStop(_) => (30, 12),
                    Dialog::Alert(_) => (25, 10),
                    Dialog::ConfigureAws => (50, 5 + self.available_profiles.len().max(1) as u16 + 1), // Header + Profiles + Footer
                    Dialog::Changelog => (70, 50),
                    Dialog::None => (0, 0),
                };
                
                let chunk_height = h * percent_y / 100;
                let available_height = chunk_height.saturating_sub(2);
                let max_scroll = content_lines.saturating_sub(available_height);

                if self.dialog == Dialog::Settings {
                    if self.settings_selected_field != SettingsField::TestSound {
                        self.navigate_settings_field(false);
                        self.ensure_dialog_selection_visible();
                    } else if self.dialog_scroll_offset < max_scroll {
                        self.dialog_scroll_offset += 1;
                    }
                } else if matches!(self.dialog, Dialog::ConfigureAws | Dialog::SessionExpired) {
                     if !self.available_profiles.is_empty() && self.selected_profile_index < self.available_profiles.len().saturating_sub(1) {
                        self.selected_profile_index += 1;
                        self.ensure_dialog_selection_visible();
                    } else if self.dialog_scroll_offset < max_scroll {
                        self.dialog_scroll_offset += 1;
                    }
                } else if self.dialog_scroll_offset < max_scroll {
                    self.dialog_scroll_offset += 1;
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
                    Dialog::Alert(_) | Dialog::Help | Dialog::Setup | Dialog::Changelog => {
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
                 if self.dialog == Dialog::SessionExpired {
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

    /// Ensure the selected item in a dialog is visible
    fn ensure_dialog_selection_visible(&mut self) {
        let (_, h) = self.window_size;
        
        // Get dialog height percentage and top padding (lines before selection)
        let (percent_y, top_padding) = match self.dialog {
            Dialog::ConfigureAws | Dialog::SessionExpired => (50, 5), // 5 header lines
            Dialog::Settings => (60, 5), // 5 header lines
            _ => return,
        };
        
        let chunk_height = h * percent_y / 100;
        // Padding: 2 (borders) + 1 (inner top padding) = 3
        let available_height = chunk_height.saturating_sub(3);
        
        // Calculate target line index
        let target_line = match self.dialog {
            Dialog::ConfigureAws | Dialog::SessionExpired => {
                top_padding + self.selected_profile_index as u16
            },
            Dialog::Settings => {
                let idx = match self.settings_selected_field {
                    SettingsField::RefreshInterval => 0,
                    SettingsField::ShowLogsPanel => 1,
                    SettingsField::LogLevel => 2,
                    SettingsField::AlertThreshold => 3,
                    SettingsField::SoundEnabled => 4,
                    SettingsField::TestSound => 5,
                };
                top_padding + (idx * 2)
            },
            _ => 0,
        };

        if target_line < self.dialog_scroll_offset {
            self.dialog_scroll_offset = target_line;
        } else if target_line >= self.dialog_scroll_offset + available_height {
            self.dialog_scroll_offset = target_line.saturating_sub(available_height).saturating_add(1);
        }
    }
}
