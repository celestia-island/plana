+++
title = "Building Guide"
description = """- [Prerequisites](#prerequisites)"""
lang = "en"
category = "guides"
subcategory = "core"
+++

# Building Guide

---

## Table of Contents

- [Prerequisites](#prerequisites)
- [Installation](#installation)
- [Configuration](#configuration)
- [Building](#building)
- [Running](#running)
- [Database management](#database-management)
- [Development environment](#development-environment)
- [Deployment](#deployment)
- [Troubleshooting](#troubleshooting)
- [Running the webhook bot](#running-the-webhook-bot)

---

## Prerequisites

### System requirements

- **OS**: Linux, macOS, or Windows (requires Docker CLI)
- **Memory**: minimum 8GB, 16GB recommended
- **Storage**: minimum 20GB free space
- **CPU**: 4 cores or more recommended

> Note (design intent)
> The core requirement on the Windows side is that Docker CLI is available; commands can be run directly in PowerShell or Windows Terminal.
> However, the containers still ultimately require a Linux runtime to host them:
> 1. The local solution is typically Docker Desktop (generally relying on a WSL2 backend).
> 2. The alternative is to install only Docker CLI on the local machine and forward to a remote Linux Docker host via `docker context`.

### Software requirements

#### Required software

- **Docker or Podman** (container runtime environment)

```bash
docker --version
docker compose version
```

Please use the officially recommended installation method for your current platform:

- Linux: install Docker Engine, Docker Desktop for Linux, or the Podman shipped by your distribution
- macOS: install Docker Desktop or Podman Desktop
- Windows: install Docker Desktop or Podman Desktop

**Important notes**:

- Runtime dependencies such as PostgreSQL are already included in the containerized environment
- However, if you want to run `just` recipes or helper scripts inside the repository, the host still needs Python 3.8+ installed
- There is no need to install PostgreSQL separately on the host
- On Windows, commands can be run directly in PowerShell or Windows Terminal, but deployment still requires an available Docker/Podman Linux runtime. Local deployment usually means using Docker Desktop with a WSL2 backend; alternatively, you can forward to a remote Linux Docker host via the local Docker CLI/context.

- **Rust 1.85+** (only needed for development builds)

```bash
rustup update stable
```

Please use the official rustup installation method for your platform:

- Linux/macOS: visit <https://rustup.rs>
- Windows: download and run `rustup-init.exe` from <https://rustup.rs>, then run `rustup update stable`

#### Recommended software

- **just** (command runner)

```bash
  # Using cargo
  cargo install just

  # Using brew (macOS)
  brew install just
  ```

- **VS Code** with the rust-analyzer extension installed

---

## Installation

### Step 1: Clone the repository

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### Step 2: Configure environment variables

```bash
# Edit the configuration after creating .env from .env.example
nano .env  # or use your preferred editor
```

Use your current shell or file manager to copy `.env.example` to `.env`.

POSIX shell:

```bash
cp .env.example .env
```

PowerShell:

```powershell
Copy-Item .env.example .env
```

#### Basic configuration

```bash
# Database configuration (configured automatically inside the container)
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# LLM quick init; import ApoRia after startup
# Single provider:
# LLM_API_KEY=your-api-key-here
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# Multiple providers (semicolon-separated):
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# Provider-level shortcut entry (recommended)
OPENAI_API_KEY=your-api-key-here
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# WebSocket configuration
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### LLM environment variable configuration notes

> **Important**: LLM provider configuration is currently managed centrally by ApoRia. Environment variables serve only as a bootstrap entry point and are no longer the long-term configuration source.

**How it works**:

1. When the TUI needs to auto-start the server, it reads the generic `LLM_*` quick-init variables, or provider-level variables such as `OPENAI_API_KEY`. Multi-provider configuration uses semicolon-separated parallel arrays: `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_PROTOCOL`, `LLM_MODEL_DEEP`, `LLM_MODEL_NORMAL`, `LLM_MODEL_BASIC`. Programming-plan environment variables (e.g. `BIGMODEL_API_KEY_CODING_PRO`) also support semicolon-separated multiple keys, auto-numbered `(#2)`, `(#3)`. Custom providers show their domain name in parentheses.
1. Before the server starts, the TUI first pre-writes the first batch of provider configurations to `res/prompts/agents/aporia/config.toml`
1. After pre-writing is done, the ApoRia configuration and the TUI's Models page take precedence
1. Existing providers with a non-empty API key are not overwritten by environment variables

**Recommended usage**:

- Use environment variables to complete the first bootstrap
- Afterwards, maintain everything via the Models page or `res/prompts/agents/aporia/config.toml`

### Step 3: Start the services

```bash
# Start all services using Docker Compose
docker compose up -d

# Or use the just command (if installed)
just dev
```

---

## Configuration

### LLM provider configuration

Entelecheia supports multiple LLM providers. Configure your preferred provider:

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### Local LLM (Ollama)

```bash
# Configure the local provider via the Models page or res/prompts/agents/aporia/config.toml
# endpoint = http://localhost:11434
# model = llama2
```

### Docker configuration

```bash
# Docker socket (usually auto-detected)
DOCKER_HOST=unix:///var/run/docker.sock

# Container settings
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## Building

### Development build

```bash
# Quick development build
just build-dev
```

### Production build

```bash
# Optimized release build
just build
```

### Building specific components

```bash
# Build only the server
cargo build -p scepter

# Build only the TUI
cargo build -p entelecheia-tui

# Build a specific agent
cargo build -p haplotes
```

### Build artifacts

After building, you will find:

- **Binaries**: in `target/debug/` or `target/release/`
- **Docker images**: built automatically during `just dev`

---

## Running

### Development mode

```bash
# Start the full development environment (including the TUI)
just dev

# Start only the server (no TUI)
just dev --no-tui

# Clean start (deletes all data)
just dev-clean
```

### Production mode

```bash
# Start the server
just server

# Start the TUI client
just tui

# Start all agents
just agents-up
```

### Terminal compatibility parameters

The TUI relies on ANSI escape sequences, mouse events, and image rendering (Sixel/Kitty protocols). In constrained terminal environments — such as SSH sessions, serial consoles, CI runners, or legacy terminal emulators — three progressive degradation parameters are available:

#### `--no-image-render`

Disables all image rendering. The other features — colors, mouse, diff refresh — remain fully functional.

```bash
just tui -- --no-image-render
```

Applicable scenarios: the terminal supports colors and mouse but lacks Sixel/Kitty image protocol support (the most common case).

#### `--no-ansi`

Disables mouse capture and special key listening. Colors and diff (partial) screen refresh are preserved. Useful when mouse events interfere with terminal selection, copy-paste, or scrollback history.

```bash
just tui -- --no-ansi
```

Applicable scenarios: you need colors but mouse capture causes problems (terminal multiplexers, `screen`, basic `tmux` configurations, etc.).

#### `--no-ansi-pure`

Pure monochrome mode — the most aggressive degradation. Disables all ANSI colors (globally forces `Color::Reset`), disables mouse capture, and performs a full-screen redraw every frame. The splash-screen logo is replaced with a pure ASCII-art version. This parameter implies `--no-ansi`.

```bash
just tui -- --no-ansi-pure
```

Applicable scenarios: running via SSH, serial consoles, `docker exec`, CI environments with minimal terminal support, or any terminal that does not handle ANSI color codes correctly.

#### Parameter comparison

| Feature | Default | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| Colors | Full | Full | Full | Disabled |
| Mouse capture | Yes | Yes | No | No |
| Image rendering | Yes | No | No | No |
| Screen refresh | Diff | Diff | Diff | Full-screen redraw |
| Startup logo | ANSI color | ANSI color | ANSI color | Pure ASCII art |

### Service management

```bash
# Check service status
just dev-status

# View logs
just dev-logs

# Stop services
just dev-down

# Force-kill all services
just dev-kill
```

---

## Database management

### Initialize the database

```bash
# Create the database
just db-create

# Run migrations
just db-migrate

# Initialize with seed data
just db-init
```

### Database operations

```bash
# Check database status
just db-status

# Back up the database
just db-backup

# Restore the database
just db-restore backups/backup_xxx.sql

# Reset the database (warning: deletes all data)
just db-reset
```

### Migration management

```bash
# Create a new migration
cargo test -p scepter test_create_migration -- --nocapture --ignored

# Roll back the last migration
just db-migrate-down
```

---

## Development environment

### Environment setup

```bash
# Initialize all dependencies
just init

# Check Python dependencies

# Format code
just fmt

# Run the linter
just clippy
```

### Testing

```bash
# Run all tests
just test

# Run a specific type of test
just test unit
just test integration
just test e2e
just test llm-providers

# Verbose output
just test verbose
```

### Code quality

```bash
# Format code
just fmt

# Check formatting
just fmt-check

# Run clippy
just clippy

# Type checking
just check
```

---

## Deployment

### Docker deployment

#### Build the image

```bash
docker build -t entelecheia:latest .
```

#### Run the container

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Docker Compose deployment

```bash
# Start all services
docker compose up -d

# View logs
docker compose logs -f

# Stop services
docker compose down
```

---

## Troubleshooting

### Common problems

#### Docker permission denied

```bash
# Add the user to the docker group
sudo usermod -aG docker $USER

# Log out and log back in
```

#### Port already in use

```bash
# Check the process using the port
lsof -i :8424

# Kill the process
kill -9 <PID>
```

#### Build failure

```bash
# Clean build artifacts
cargo clean

# Update dependencies
cargo update

# Rebuild
just build
```

#### Container fails to start

```bash
# Check Docker logs
docker compose logs

# Rebuild the container
docker compose down
docker compose build --no-cache
docker compose up -d
```

### Getting help

1. Search [GitHub Issues](https://github.com/celestia-island/entelecheia/issues)
1. Join our [discussions](https://github.com/celestia-island/entelecheia/discussions)

---

## Running the webhook bot

The webhook bot lives under `plugins/github-webhook/`. Each platform has its own directory.

### Prerequisites

- Python 3.10+ (current bots)
- Node.js 18+ (for the future TypeScript migration)
- Bot tokens for each platform (see the [Webhook configuration guide](webhook-setup.md))

### Running a single bot

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### Running all bots

```bash
just webhooks-up
```

### Environment variables

Copy the example environment file and configure it:

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

For platform-specific configuration details, see the [Webhook configuration guide](webhook-setup.md).

---

## Next steps

- Read the [Fundamentals guide](fundamentals.md) to understand the architecture
- Browse the [agent documentation](../../agents/) to learn about available agents

---

**Happy building!** 🚀
