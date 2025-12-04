# Jumpstart - Windows Application Launcher

A modern Windows application that automatically launches and positions multiple applications on specific monitors according to a YAML configuration file.

## What It Does

- Launch multiple applications simultaneously
- Position windows on specific monitors (left/right sides)

## How it works

- Monitor Detection: The application uses Windows API to enumerate all connected monitors and their dimensions
- Application Launching: Applications are launched using the Windows shell start command
- Window Detection: The application searches for windows by title (case-insensitive partial match)
- Window Positioning: Windows are positioned using SetWindowPos API with calculated coordinates

## Quick Start

### Install from Source

```bash
cd jumpstart
cargo build --release --features embedded_config
```

### Run

**GUI Mode (Default)**:
```bash
jumpstart.exe
```

**CLI Mode**:
```bash
jumpstart.exe --cli
```

## Configuration

The application works out-of-the-box with an embedded default configuration. Use the GUI editor to modify configurations or create your own `config.yml`:

```yaml
applications:
  - name: "App Name"
    display: 1
    side: "left"
    executable: "path/to/app.exe"
```

## CLI Options

```bash
jumpstart.exe --help
```

- `-f, --config <FILE>`: Configuration file (default: config.yml)
- `-c, --cli`: Launch in CLI mode instead of GUI
- `-h, --help`: Show help
