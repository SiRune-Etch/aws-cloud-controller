# AWS Cloud Controller

A powerful terminal-based interface (TUI) for managing AWS resources (EC2, Lambda) directly from your command line.

## Features

- **EC2 Management**: List, Start, Stop, and Terminate instances.
- **Auto-Stop Scheduling**: Schedule instances to stop automatically after a duration (save costs!).
- **Alerts**: Get visual and audio alerts for long-running instances.
- **Responsive UI**: Adapts to different terminal sizes with mouse support and scrolling.
- **Cross-Platform**: Works on macOS, Linux, and Windows.

## Installation

### Prerequisites

1.  **Rust Toolchain**: Install via [rustup.rs](https://rustup.rs).
2.  **AWS Credentials**: Configure using `aws configure` or environment variables.
3.  **Platform-Specific Dependencies**:

    **Linux (Ubuntu/Debian)**:
    Required for audio support (`rodio`).

    ```bash
    sudo apt-get install libasound2-dev pkg-config
    ```

    **Windows**:
    No additional dependencies required. Audio support works out of the box.
    Recommended terminals: Windows Terminal, PowerShell, or CMD.

    **macOS**:
    No additional dependencies required.

### Install from Source

You can install the binary directly from the repository using `cargo`:

```bash
cargo install --git https://github.com/SiRune-Etch/aws-cloud-controller
```

This will compiled the project in release mode and install the `aws-cloud-controller` binary to your `~/.cargo/bin` folder.

Ensure `~/.cargo/bin` is in your `PATH`.

## Usage

Simply run the application:

```bash
aws-cloud-controller
```

### Controls

- **Navigation**: `↑`/`↓` or `j`/`k`
- **Switch Tabs**: `1` (Home), `2` (EC2), `3` (Lambda), `4` (About), `5` (Logs)
- **Actions**:
  - `s`: Start Instance
  - `x`: Stop Instance
  - `t`: Terminate Instance
  - `a`: Schedule Auto-Stop
  - `r`: Refresh Data
- **General**:
  - `?` or `h`: Help
  - `,`: Settings
  - `q`: Quit

## Configuration

Settings are stored in platform-appropriate locations:

- **Linux**: `$XDG_CONFIG_HOME/aws-cloud-controller/settings.json` (or `~/.config/aws-cloud-controller/`)
- **macOS**: `~/Library/Application Support/aws-cloud-controller/settings.json`
- **Windows**: `%APPDATA%\aws-cloud-controller\settings.json`

You can configure:

- Auto-refresh interval (15s, 30s, 60s, 120s, 300s)
- Alert threshold for long-running instances
- Sound alerts (enable/disable)
- Logs panel visibility
