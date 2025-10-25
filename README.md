# ag - agnostic' magic is now at your fingertips.

A powerful command-line tool for managing projects and pipelines with automatic binary management.

## Features

- **Project Management**: Initialize and manage projects with ease
- **Pipeline Operations**: Spawn and control data pipelines with S3 integration
- **Automatic Binary Management**: Automatically downloads and manages required binaries
- **System Status Monitoring**: Check the health and status of all components
- **Cross-Platform Support**: Works on macOS (ARM64/x86_64) and Linux (x86_64)

## Requirements

- Rust 1.70+ (2024 edition)
- Internet connection (for downloading managed binaries)
- Supported platforms:
  - macOS (ARM64/x86_64)
  - Linux (x86_64)

## Installation

### From Source

```bash
git clone <repository-url>
cd cli
cargo build --release
cp target/release/ag /usr/local/bin/  # or any directory in your PATH
```

### Quick Start

```bash
# Check system status
ag status system

# View binary status
ag status binaries

# Initialize a new project
ag project init my-project

# Get project information
ag project info

# Spawn a pipeline
ag pipeline spawn

# Get pipeline information
ag pipeline info

# Check version
ag --version
```

## Commands

### Project Management

```bash
ag project init <name>    # Initialize a new project
ag project info           # Get information about a project
```

### Pipeline Management

```bash
ag pipeline spawn         # Spawn a new pipeline with S3 server
ag pipeline info          # Get information about a pipeline
```

### System Status

```bash
ag status binaries        # Show status of all installed binaries
ag status system          # Show overall system status
```

## Managed Binaries

The tool automatically manages the following binaries:

### s3fs
- **Purpose**: S3 server with filesystem backend
- **Source**: [agnosticeng/s3fs](https://github.com/agnosticeng/s3fs)
- **Platforms**: macOS (ARM64/x86_64), Linux (x86_64)

### ClickHouse
- **Purpose**: High-performance columnar database
- **Source**: Official ClickHouse releases
- **Platforms**: macOS (ARM64/x86_64), Linux (x86_64)

### agt
- **Purpose**: Agnostic toolkit binary
- **Source**: [agnosticeng/agt](https://github.com/agnosticeng/agt)
- **Platforms**: macOS (ARM64/x86_64), Linux (x86_64)

## Directory Structure

The tool creates and manages a working directory at `~/.agnostic/`:

```
~/.agnostic/
├── bin/           # Downloaded binaries
│   ├── s3fs       # S3 server binary
│   ├── clickhouse # ClickHouse database binary
│   └── agt        # Agnostic toolkit binary
├── cache/         # Cache files
├── logs/          # Log files
├── projects/      # Project data
└── temp/          # Temporary files
```

## Binary Installation

Binaries are automatically downloaded and installed on first use:

- Downloaded from official GitHub releases
- Saved to `~/.agnostic/bin/` with executable permissions
- Platform-specific binaries are automatically selected
- Installation progress is shown with download size and speed
- Binary integrity is verified after installation

### Installation Output

When binaries need to be installed, you'll see:

```
Installing s3fs binary...
Downloading s3fs binary for macosaarch64...
Download size: 6291456 bytes (6.00 MB)
Download completed: 6291456 bytes
s3fs binary installed successfully at: /Users/username/.agnostic/bin/s3fs
Verifying s3fs binary...
s3fs version: v0.0.1 (from agnosticeng/s3fs)
Binary setup completed: 1 new binaries installed
```

If all binaries are already installed, the tool runs silently without any installation messages.

## Verbose Mode

For detailed output, use the verbose flag or environment variable:

```bash
# Using environment variable
VERBOSE=1 ag status system

# Using command line flag (if supported)
ag --verbose status system
```

## Platform Support

- **macOS ARM64** (Apple Silicon)
- **macOS x86_64** (Intel)
- **Linux x86_64**

The tool automatically detects your platform and downloads the appropriate binaries.

## Troubleshooting

### Binary Installation Issues

If you encounter problems with binary installation:

1. Check your internet connection
2. Verify you have write permissions to `~/.agnostic/bin/`
3. Check available disk space
4. Try removing the binary and re-running: `rm ~/.agnostic/bin/<binary-name>`

### Permission Denied

If you get permission errors:

1. Check file permissions: `ls -la ~/.agnostic/bin/`
2. Make binaries executable: `chmod +x ~/.agnostic/bin/*`
3. Ensure your user has write access to the home directory

### Binary Not Found

If a binary is reported as missing:

1. Check if the binary exists: `ls -la ~/.agnostic/bin/`
2. Verify executable permissions
3. Try forcing a re-download by deleting the binary

## Development

### Building from Source

Prerequisites:
- Rust toolchain (install from [rustup.rs](https://rustup.rs))
- Git

```bash
# Install Rust if you haven't already
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone <repository-url>
cd cli
cargo build --release
```

### Dependencies

The project uses the following main dependencies:
- `clap` - Command line argument parsing
- `reqwest` - HTTP client for downloading binaries
- `tokio` - Async runtime
- `indicatif` - Progress bars for downloads
- `zip` - Archive extraction

### Running Tests

```bash
cargo test
```

### Project Structure

- `src/main.rs` - Main entry point and CLI argument parsing
- `src/commands/` - Command implementations (project, pipeline, status)
- `src/utils/` - Utility modules
  - `src/utils/app/` - Application initialization and configuration
  - `src/utils/bin/` - Binary management and providers
  - `src/utils/fs/` - Filesystem utilities
  - `src/utils/net/` - Network and download utilities

## Configuration

The tool uses sensible defaults but can be configured through environment variables:

- `VERBOSE=1` - Enable verbose output
- `AG_HOME` - Override the default `~/.agnostic` directory (optional)

## License

[Add your license information here]

## Contributing

[Add contribution guidelines here]

## Support

If you encounter issues:

1. Check the [troubleshooting section](#troubleshooting)
2. Run `ag status system` to diagnose problems
3. Use verbose mode for detailed output: `VERBOSE=1 ag <command>`
4. Check binary status: `ag status binaries`

For additional help, please open an issue in the repository.
