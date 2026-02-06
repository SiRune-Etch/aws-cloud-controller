# Changelog

All notable changes to this project will be documented in this file.

## [0.2.1] - 2026-02-07

### Added

- **Async Profile Switching**: Profile activation is now non-blocking and runs in the background.
- **UI Polish**:
  - **Loading Overlay**: Visual feedback ("Processing...") during profile switches.
  - **Active Profile Indicator**: Green checkmark (✅) shows the currently active profile.
- **Fixes**:
  - Fixed "Session Expired" dialog not closing automatically.
  - Fixed Footer layout text overlapping.
  - Fixed Default Profile not being highlighted on startup.

## [0.2.0] - 2026-02-07

### Added

- **AWS SSO Integration**: Seamless support for AWS IAM Identity Center (SSO).
- **Profile Switching**:
  - Interactive profile selector in `ConfigureAws` and `SessionExpired` dialogs.
  - Explicit profile activation via `Enter` key.
  - Auto-activation after successful SSO login.
- **Improved Error Handling**:
  - Captures `aws sso login` output and errors.
  - Displays cleaner, user-friendly Toast notifications.
  - Handles "Session Expired" states with a dedicated recovery dialog.
- **UI Enhancements**:
  - "Active Profile" status messages.
  - Cleaned up dialog text and instructions.

### Fixed

- Fixed an issue where the `default` profile was being used despite SSO login attempts.
- Fixed raw process output corruption on the TUI screen.
- Fixed missing profile persistence during runtime sessions.

## [0.1.0] - Initial Release

- Basic EC2 and Lambda management.
- TUI Dashboard.
- Log viewer.
