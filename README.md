# Jumpstart - Windows Application Launcher and Window Manager

[![Tests](https://github.com/ShawonAshraf/jumpstart/actions/workflows/test.yml/badge.svg)](https://github.com/ShawonAshraf/jumpstart/actions/workflows/test.yml)

Jumpstart is a Rust application that automatically launches and positions multiple applications on Windows 11 according to a YAML configuration file. It's designed to help you quickly set up your workspace by launching applications and positioning them on specific monitors with split-screen configurations.

## Features

- Launch multiple applications automatically
- Position applications on specific monitors
- Split monitor screen between left and right sides
- Configurable via YAML file
- Uses Windows APIs for reliable window management

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
  - name: Microsoft Teams
    display: 2
    side: right
    executable: "path to teams.exe"
  - name: Outlook
    display: 2
    side: left
    executable: "path to outlook.exe"
  - name: Slack
    display: 3
    side: right
    executable: "path to slack.exe"
  - name: Notion
    display: 3
    side: left
    executable: "path to notion.exe"
```

### Configuration Fields

- `name`: Display name of the application (used for logging and window detection)
- `display`: Monitor number (1-based indexing, where 1 is the primary display)
- `side`: Side of the monitor to position the application (`left` or `right`)
- `executable`: Name of the executable file (must be in PATH or provide full path)

## Usage

### Basic Usage

1. Configure your applications in `config.yml`
2. Run the application:
   ```bash
   .\target\release\jumpstart.exe
   ```

### Using a Custom Configuration File

You can specify a custom configuration file using the `-c` or `--config` flag:

```bash
# Using short flag
.\target\release\jumpstart.exe -c path/to/your/config.yml

# Using long flag
.\target\release\jumpstart.exe --config path/to/your/config.yml
```

If no config file is specified, the application will look for `config.yml` in the current directory.

## How It Works

1. **Monitor Detection**: The application uses Windows API to enumerate all connected monitors and their dimensions
2. **Application Launching**: Applications are launched using the Windows shell `start` command
3. **Window Detection**: The application searches for windows by title (case-insensitive partial match)
4. **Window Positioning**: Windows are positioned using `SetWindowPos` API with calculated coordinates

## Troubleshooting

### Monitor Detection Issues

- Monitor numbers are 1-based (1 = primary display)
- Use Windows Display Settings to identify your monitor numbers
- The application splits the monitor work area (excluding taskbar)

## Building from Source

```bash
cargo build --release
```
