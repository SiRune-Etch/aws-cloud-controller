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
3.  **Linux Dependencies (Ubuntu/Debian)**:
    Required for audio support (`rodio`).
    ```bash
    sudo apt-get install libasound2-dev pkg-config
    ```

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
- **Switch Tabs**: `1` (Home), `2` (EC2), `3` (Lambda), `4` (About)
- **Actions**:
  - `s`: Start Instance
  - `x`: Stop Instance
  - `t`: Terminate Instance
  - `a`: Schedule Auto-Stop
  - `r`: Refresh Data
- **General**:
  - `?` or `h`: Help
  - `q`: Quit

## License

MIT
