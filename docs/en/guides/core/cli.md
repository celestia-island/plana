+++
title = "CLI Usage Guide"
description = """`entelecheia-cli` is the command-line interface for the Entelecheia multi-agent collaboration platform. It communicates with the scepter server over a Unix socket JSON-RPC, providing chat interaction, service lifecycle management, agent control, configuration, and more."""
lang = "en"
category = "guides"
subcategory = "core"
+++

# CLI Usage Guide

`entelecheia-cli` is the command-line interface for the Entelecheia multi-agent collaboration platform. It communicates with the scepter server over a Unix socket JSON-RPC, providing chat interaction, service lifecycle management, agent control, configuration, and more.

> Note: The CLI does not yet reach full feature parity with the TUI. For the current status, see [ARCHITECTURE.md](../../ARCHITECTURE.md).

---

## Table of Contents

- [Installation](#installation)
- [Basic usage](#basic-usage)
- [Global options](#global-options)
- [Chat commands](#chat-commands)
- [Agent management](#agent-management)
- [Service lifecycle](#service-lifecycle)
- [Configuration](#configuration)
- [Connection context](#connection-context)
- [Status and monitoring](#status-and-monitoring)
- [Subscriptions (Layer3)](#subscriptions-layer3)
- [Running agents](#running-agents)
- [Timeline](#timeline)
- [Docker images](#docker-images)
- [Advanced usage](#advanced-usage)

---

## Installation

### Build from source

```bash
# Clone the repository
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# Build the CLI binary
cargo build --package entelecheia-cli

# Or use just
just cli
```

The binary is located at `target/debug/entelecheia-cli` (debug) or `target/release/entelecheia-cli` (release).

### Pre-built binaries

Pre-built binaries are available from [GitHub Releases](https://github.com/celestia-island/entelecheia/releases). Download the archive for your platform and place the binary in your `PATH`.

---

## Basic usage

```bash
# Show help
entelecheia-cli --help

# Send a message through the skill chain
entelecheia-cli send explain the architecture of this project

# Send a message via a pipe
echo "summarize this file" | entelecheia-cli send

# Check system status
entelecheia-cli status
```

---

## Global options

| Option | Description | Default |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | Log level (trace, debug, info, warn, error) | `warn` |
| `-d, --daemon` | Dispatch the command in the background and exit immediately | — |
| `-c, --clean` | Clean up Cosmos containers and socket files | — |
| `-a, --auto-approve` | Auto-approve operations (make sure the server is running) | — |
| `-t, --table` | Human-readable table output (ANSI formatted) | Default |
| `-j, --json` | JSON output (machine-readable) | — |
| `-r, --raw` | Raw plain-text output (no formatting) | — |
| `--format <FORMAT>` | Output format (table, json, raw) | `table` |

Output format options:

- `table` — human-readable table output
- `json` — machine-readable JSON output

**Examples:**

```bash
# Clean up containers
entelecheia-cli --clean

# Get status as JSON
entelecheia-cli status --format json

# Send a message in debug mode
entelecheia-cli -l debug send "debug connection issue"

# Run an agent in the background (returns immediately)
entelecheia-cli -d run my-agent --ci
```

---

## Chat commands

The `chat` subcommand manages conversational interaction with the session agent system.

### Send a message

```bash
entelecheia-cli chat send [OPTIONS]
```

| Option | Description |
| --- | --- |
| `-m, --message <MSG>` | The message text to send |
| `--stdin` | Read the message from standard input |
| `-f, --file <PATH>` | Read the message from a file |

Only one input source can be used at a time.

**Examples:**

```bash
# Send a message directly
entelecheia-cli chat send -m "Hello, what can you do?"

# From standard input
echo "analyze the code in src/main.rs" | entelecheia-cli chat send --stdin

# From a file
entelecheia-cli chat send -f ./prompts/review.txt
```

The `chat send` command sends the message through the **skill chain** — the core execution pipeline that coordinates multiple agents. Progress is shown via a spinner animation during execution.

### Chat history

```bash
entelecheia-cli chat history [OPTIONS]
```

| Option | Description | Default |
| --- | --- | --- |
| `--conversation <ID>` | Filter by conversation ID | — |
| `--agent <TYPE>` | Filter by agent type | — |
| `--role <ROLE>` | Filter by role (user/assistant/system) | — |
| `--from <ISO8601>` | Start date-time (ISO 8601) | — |
| `--to <ISO8601>` | End date-time (ISO 8601) | — |
| `--limit <N>` | Maximum number of messages to return | `50` |
| `--offset <N>` | Pagination offset | `0` |

**Example:**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### Recent messages

```bash
entelecheia-cli chat recent [OPTIONS]
```

| Option | Description | Default |
| --- | --- | --- |
| `--timeline <ID>` | Filter by timeline/conversation ID | — |
| `--agent <TYPE>` | Filter by agent type | — |
| `--limit <N>` | Maximum number of messages to return | `20` |

---

## Agent management

Manage the agent lifecycle (list, start, stop, restart).

```bash
entelecheia-cli agent <COMMAND>
```

### Commands

```bash
# List all agents and their state
entelecheia-cli agent list

# Start an agent by type
entelecheia-cli agent start <AGENT_TYPE>

# Stop a running agent
entelecheia-cli agent stop <AGENT_TYPE>

# Restart an agent
entelecheia-cli agent restart <AGENT_TYPE>
```

**Available agent types:** ApoRia, EleOs, EpieiKeia, Haplotes, HubRis, Kalos, NeiKos, OreXis, PhiLia, Polemos, SkeMma, SkoPeo.

> Note: Agents run as library crates inside the scepter runtime, not as standalone executables. The `agent start` command attempts to spawn a binary matching the agent's name, which is mainly applicable when an agent is compiled as a separate binary. In practice, agents are activated through the scepter server.

---

## Service lifecycle

Manage the Entelecheia service stack with Docker containers.

### Initialize services

```bash
entelecheia-cli init [OPTIONS]
```

Sets up the full service stack: PostgreSQL (with pgvector), the Docker registry, the scepter server, and the WebUI. It creates the required Docker networks and pulls/builds the images.

| Option | Description | Default |
| --- | --- | --- |
| `--prefix <STR>` | Container name prefix | `e-` |
| `--source-build` | Build images from source instead of pulling | `false` |
| `--webui-port <PORT>` | WebUI port | `3424` |

**Example:**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### Start all services

```bash
entelecheia-cli serve [OPTIONS]
```

Starts all previously initialized containers. Requires `init` to have been run first.

| Option | Description | Default |
| --- | --- | --- |
| `--prefix <STR>` | Container name prefix | `e-` |
| `--webui-port <PORT>` | WebUI port | `3424` |

### Stop all services

```bash
entelecheia-cli stop [OPTIONS]
```

Stops all running containers in order: webui → scepter → registry → postgres.

| Option | Description | Default |
| --- | --- | --- |
| `--prefix <STR>` | Container name prefix | `e-` |

### Start WebUI only

```bash
entelecheia-cli webui [OPTIONS]
```

Starts or creates only the WebUI container.

| Option | Description | Default |
| --- | --- | --- |
| `--prefix <STR>` | Container name prefix | `e-` |
| `--webui-port <PORT>` | WebUI port | `3424` |

---

## Configuration

View and validate system configuration.

### Show configuration

```bash
entelecheia-cli config show
```

Shows the current configuration, including:

- Database URL and connection settings
- ApoRia LLM provider configuration (name, model, endpoint)
- WebSocket bind address
- Log level

API keys are masked in the output (shown as `***`).

### Validate configuration

```bash
entelecheia-cli config validate
```

Performs validation checks:

- Database URL is set
- At least one ApoRia provider is configured with complete settings
- WebSocket bind address is set

Returns a pass/fail result with details about any issues.

**Example output:**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## Connection context

The `context` subcommand manages named connection profiles, letting you switch between local (Unix socket) and remote (WebSocket) scepter servers. It works similarly to Docker's `docker context` command.

### Concepts

A **context** is a named profile that records how the CLI connects to the scepter server:

- **local** — Unix socket connection (default, auto-resolved to `/run/.../entelecheia-tui.sock`)
- **remote** — WebSocket connection with Bearer token authentication

Contexts are stored in `~/.config/entelecheia/contexts/contexts.toml`.

### List contexts

```bash
entelecheia-cli context list
```

The currently active context is marked with `*`.

### Show the current context

```bash
entelecheia-cli context show
```

Shows the type, socket path, WS URL, and description of the active context.

### Create a context

```bash
# Remote WebSocket context
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Staging server"

# An additional local context
entelecheia-cli context create dev --description "Development server"
```

To obtain a Bearer token from a remote server:

```bash
# On the server machine
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### Switch context

```bash
entelecheia-cli context use staging
# From now on, all commands (send, status, chat, etc.) will be routed through staging
```

### Remove a context

```bash
entelecheia-cli context remove staging
```

The `default` context cannot be removed.

### Example workflow

```bash
# View current contexts
entelecheia-cli context list

# Create a remote context for the staging server
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# Switch to staging
entelecheia-cli context use staging

# Send a message through the remote server
entelecheia-cli send "list current to-do items"

# Check the remote server status
entelecheia-cli status

# Switch back to local
entelecheia-cli context use default
```

---

## Status and monitoring

### System status

```bash
entelecheia-cli status
```

Shows:

- Server version
- Connection state (socket status)
- LLM provider summary
- WebSocket bind address
- Agent list with running/stopped status
- System resources (memory usage, load average)

### Status path query

The `status` command accepts a path-like argument to query specific subsystems. The syntax supports agent-scoped timelines, chat history inspection, and device enumeration.

```bash
entelecheia-cli status <PATH> [--raw]
```

| Path syntax | Description |
| --- | --- |
| `timeline.#agent[-N]` | Show the most recent N skill-invocation records of an agent |
| `timeline.#agent[N][M]` | Show the M-th MCP/tool call within the N-th skill invocation |
| `history[-N]` | Show the most recent N chat messages (all roles) |
| `history[-N].body` | Show the body of the Nth-from-last message |
| `device` | List all edge devices recognized by Polemos |
| `device[N]` | Show details of the N-th Polemos device |

**Examples:**

```bash
# The 30 most recent skill-scheduling records for the Haplotes #001 agent
entelecheia-cli status timeline.#hap_lotes.001[-30]

# The 2nd MCP/tool call in the 3rd skill invocation
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# The 30 most recent messages
entelecheia-cli status history[-30]

# The body of the 3rd-from-last message (plain text)
entelecheia-cli status history[-3].body --raw

# All Polemos devices
entelecheia-cli status device

# Details of the 3rd Polemos device
entelecheia-cli status device[3]
```

> **Shell note:** In bash/zsh, wrap paths containing `[...]` in single quotes to prevent glob expansion: `entelecheia-cli status 'history[-30]'`. The `#` character does not need escaping when embedded in the middle of a word. In fish shell, none of these paths require quotes.

Status path queries communicate with the server over a Unix socket JSON-RPC. The `timeline.*` and `history.*` queries require the server to be running. The `device` query requires a Polemos workspace to be registered on the server.

### View logs

```bash
entelecheia-cli logs [OPTIONS]
```

| Option | Description | Default |
| --- | --- | --- |
| `-a, --agent <NAME>` | Filter logs by agent name | All agents |
| `-l, --lines <N>` | Number of lines to show (tail) | `100` |

**Examples:**

```bash
# Show the last 200 lines of logs for all agents
entelecheia-cli logs -l 200

# Show ApoRia logs
entelecheia-cli logs -a ApoRia
```

Logs are read from the `./logs/` directory. Each agent has its own log file (`ApoRia.log`, `EleOs.log`, etc.).

---

## Subscriptions (Layer3)

Manage Layer3 agent subscriptions — external agent packages that can be installed and run.

### List subscriptions

```bash
entelecheia-cli subscribe list
```

Shows all configured subscriptions, including state (installed/pending), enabled status, auto-update setting, and source.

### Add a subscription

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| Option | Description |
| --- | --- |
| `--name <NAME>` | Subscription name (required) |
| `--source <SOURCE>` | Source type: `official`, `github`, or `url` (required) |
| `--repository <REPO>` | GitHub repository (for github sources) |
| `--url <URL>` | Direct URL (for url sources) |
| `--version <VER>` | Version constraint |
| `--auto-update` | Enable auto-update |
| `--disabled` | Add it in a disabled state |

**Example:**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### Remove a subscription

```bash
entelecheia-cli subscribe remove <NAME>
```

### Sync subscriptions

```bash
# Sync all subscriptions
entelecheia-cli subscribe sync

# Sync a specific subscription
entelecheia-cli subscribe sync --name my-agent
```

### Auto-update

```bash
entelecheia-cli subscribe auto-update
```

Updates all subscriptions that have `auto_update` enabled.

---

## Running agents

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

Runs a Layer3 agent script. It looks for `.amphoreus/<AGENT>/run.py` in the current directory. On first execution, it runs a pre-check audit.

| Option | Description |
| --- | --- |
| `--ci` | Enable CI mode |
| `--auto-pr` | Enable auto-PR mode |
| `--dry-run` | Dry run (no actual changes) |
| `--providers <LIST>` | Comma-separated list of providers |
| `--output-dir <DIR>` | Output directory |

**Examples:**

```bash
# Run a Layer3 agent in dry-run mode
entelecheia-cli run my-agent --dry-run

# Run with specified providers
entelecheia-cli run my-agent --providers openai,anthropic

# CI mode with automatic PR submission
entelecheia-cli run my-agent --ci --auto-pr

# Run in background mode (returns immediately; child runs detached)
entelecheia-cli -d run my-agent --ci --auto-pr
```

### Background mode (`-d` / `--daemon`)

The background-mode flag causes the CLI to re-spawn a detached child process with the `--daemon` argument stripped, and returns immediately. The child process inherits the original command and runs independently. You can check progress afterwards with `status`.

Applicable to long-running operations such as `run`, `init`, `deploy`:

```bash
# Dispatch an agent run in the background
entelecheia-cli -d run my-agent

# Dispatch service initialization in the background
entelecheia-cli -d init --prefix prod-

# Check status later
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## Timeline

View session timelines.

### List timelines

```bash
entelecheia-cli timeline list [OPTIONS]
```

| Option | Description | Default |
| --- | --- | --- |
| `--agent <TYPE>` | Filter by agent type | — |
| `--limit <N>` | Maximum number of results | `50` |
| `--offset <N>` | Pagination offset | `0` |

### Show timeline details

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| Option | Description | Default |
| --- | --- | --- |
| `--include-messages` | Include messages in the output | `true` |

---

## Docker images

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

Builds or pulls the Docker images required by the platform.

| Option | Description |
| --- | --- |
| `--source-build` | Build images from source instead of pulling |
| `--tag <TAG>` | Image tag (default: `latest`) |

**Examples:**

```bash
# Build all images from source
entelecheia-cli init-docker-images --source-build

# Pull with a custom tag
entelecheia-cli init-docker-images --tag v0.2.0
```

Managed images:

- `entelecheia` — the orchestration server (with the embedded cosmos runtime)
- `pgvector/pgvector` — PostgreSQL with the vector extension

---

## Advanced usage

### JSON output for scripting

Use `--format json` to get machine-readable output, which can be piped to `jq` or other tools:

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### Chain cleanup and initialization

```bash
# Full teardown and rebuild
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### Debug mode

```bash
# Enable trace-level logging for debugging
entelecheia-cli -l trace send "test message"
```

### Using alongside the TUI

The CLI and the TUI connect to the same scepter server. Both can be used simultaneously:

- Start the TUI for interactive sessions: `cargo run --bin entelecheia-tui`
- Use the CLI for scripting, automation, and quick queries

---

## Troubleshooting

### "No command specified"

Run `--help` to see available commands, or use `send "message"` to send a message quickly.

### "Failed to connect to Docker"

Make sure Docker (or Podman) is running:

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

Agents are internal library crates of the scepter runtime, not standalone binaries. Start the scepter server to activate agents:

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

Set the ApoRia provider configuration via environment variables. For provider setup instructions, see the [Building guide](building.md).

### "Configuration validation failed"

Run `entelecheia-cli config validate` to see which checks failed. Common issues:

- Missing `DATABASE_URL` environment variable
- Incomplete ApoRia provider setup (name, model, `api_key`)
- Missing WebSocket bind address
