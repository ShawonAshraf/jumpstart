# Jumpstart - Windows Application Launcher and Window Manager

[![Tests](https://github.com/ShawonAshraf/jumpstart/actions/workflows/test.yml/badge.svg)](https://github.com/ShawonAshraf/jumpstart/actions/workflows/test.yml)

Jumpstart is a Rust application that automatically launches and positions multiple applications on Windows 11 according to a YAML configuration file. It's designed to help you quickly set up your workspace by launching applications and positioning them on specific monitors with split-screen configurations.

## Features

- Launch multiple applications automatically
- Position applications on specific monitors
- Split monitor screen between left and right sides
- Configurable via YAML file
- Uses Windows APIs for reliable window management

## Requirements

- Windows 11
- Rust 1.70+ (for building from source)
- Applications specified in config.yml must be installed and accessible via PATH

## Installation

### From Source

1. Clone this repository:
   ```bash
   git clone <repository-url>
   cd jumpstart
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. The executable will be available at `target/release/jumpstart.exe`

## Configuration

The application is configured via `config.yml` in the same directory as the executable. Here's an example configuration:

```yaml
applications:
  - name: Teams
    display: 2
    side: right
    executable: "teams.exe"
  - name: Outlook
    display: 2
    side: left
    executable: "outlook.exe"
  - name: Slack
    display: 3
    side: right
    executable: "slack.exe"
  - name: Notion
    display: 3
    side: left
    executable: "notion.exe"
```

### Configuration Fields

- `name`: Display name of the application (used for logging and window detection)
- `display`: Monitor number (1-based indexing, where 1 is the primary display)
- `side`: Side of the monitor to position the application (`left` or `right`)
- `executable`: Name of the executable file (must be in PATH or provide full path)

## Usage

1. Configure your applications in `config.yml`
2. Run the application:
   ```bash
   jumpstart.exe
   ```
3. The application will:
   - Read the configuration
   - Detect all connected monitors
   - Launch each application sequentially
   - Wait for applications to start (5 seconds per application)
   - Position each application on the specified monitor and side
   - Split the monitor screen evenly between left and right applications

## How It Works

1. **Monitor Detection**: The application uses Windows API to enumerate all connected monitors and their dimensions
2. **Application Launching**: Applications are launched using the Windows shell `start` command
3. **Window Detection**: The application searches for windows by title (case-insensitive partial match)
4. **Window Positioning**: Windows are positioned using `SetWindowPos` API with calculated coordinates

## Troubleshooting

### Application Not Found

If an application doesn't launch:
- Ensure the executable is in your system PATH
- Or provide the full path to the executable in the config
- Verify the application is properly installed

### Window Not Positioned

If a window launches but isn't positioned correctly:
- The application uses window title matching. Some applications might have different window titles
- You can modify the window title mapping in the source code (in the `main` function)
- Increase the wait time if applications take longer to start

### Monitor Detection Issues

- Monitor numbers are 1-based (1 = primary display)
- Use Windows Display Settings to identify your monitor numbers
- The application splits the monitor work area (excluding taskbar)

## Building from Source

Ensure you have Rust installed, then run:

```bash
cargo build --release
```

## Dependencies

- `serde`: YAML serialization/deserialization
- `serde_yaml`: YAML parsing
- `winapi`: Windows API bindings
- `widestring`: Windows wide string handling

## License

This project is open source. See the LICENSE file for details.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
