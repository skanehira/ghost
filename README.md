![GitHub Repo stars](https://img.shields.io/github/stars/skanehira/ghost?style=social)
![GitHub](https://img.shields.io/github/license/skanehira/ghost)
![GitHub all releases](https://img.shields.io/github/downloads/skanehira/ghost/total)
![GitHub CI Status](https://img.shields.io/github/actions/workflow/status/skanehira/ghost/ci.yaml?branch=main)
![GitHub Release Status](https://img.shields.io/github/v/release/skanehira/ghost)

# Ghost

Ghost is a simple background process manager for Unix systems (Linux, macOS, BSD).

<div align="center">
  <img src="./images/logo.png" width=300 />
</div>

Ghost was inspired by projects like [pueue](https://github.com/Nukesor/pueue) and [task-spooler](https://github.com/justanhduc/task-spooler).

## Features

![](./images/ghost.png)

- Run background commands without a resident daemon
- Terminal UI for monitoring, rerunning, and inspecting tasks
- Automatic log capture with live tailing
- Listening-port detection when `lsof` is available
- MCP server mode for AI assistant integration
- Works anywhere Unix process management is available

## Requirements

- Unix-based system (Linux, macOS, BSD)
- Rust 1.80+ (2024 edition)
- `lsof` (optional, used for port detection)

## Installation

### Build from source

```sh
git clone https://github.com/skanehira/ghost.git
cd ghost
cargo build --release
```

The compiled binary is written to `target/release/ghost`.

### Download prebuilt binaries

If you prefer not to build from source, download the latest prebuilt binaries from the [GitHub Releases](https://github.com/skanehira/ghost/releases) page.

## Quick Start

```bash
# Run a command in the background
ghost run echo "Hello, Ghost"

# List managed tasks
ghost list

# Inspect process output
ghost log -f <task_id>

# Run as a mcp server
ghost mcp

# Open the TUI dashboard
ghost
```

## Documentation

- [Detailed usage guide](docs/usage.md)
- [Architecture overview](docs/architecture.md)
