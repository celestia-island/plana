+++
title = "CLI Logging Conventions"
description = """The shittim-chest CLI wrapper's log output follows conventions consistent with entelecheia, using the `tracing` ecosystem, outputting to stderr in a compact human-readable format."""
lang = "en"
category = "design"
subcategory = "webui"
+++

# CLI Logging Conventions

## Overview

The shittim-chest CLI wrapper's log output follows conventions consistent with entelecheia, using the `tracing` ecosystem, outputting to stderr in a compact human-readable format.

## Framework Selection

| Component | Choice | Reason |
| --- | --- | --- |
| Logging framework | `tracing` | Rust ecosystem standard, consistent with entelecheia |
| Subscriber | `tracing-subscriber` fmt layer | Compact output, no JSON parsing needed |
| Time format | `ShortTimer` (HH:MM:SS) | Terminal-friendly, consistent with entelecheia CLI |
| Output target | stderr | Separated from stdout, does not interfere with pipes |

## Initialization Code

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// Initialization
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // hide module paths
    .with_timer(ShortTimer)      // HH:MM:SS format
    .compact()                   // compact mode
    .with_writer(std::io::stderr) // output to stderr
    .init();
```

## Format Comparison

| Mode | Example Output | Use Case |
| --- | --- | --- |
| CLI (current) | `14:23:05  INFO creating network shittim-chest...` | Development, operations |
| Server (future) | `{"timestamp":"...","level":"INFO","message":"..."}` | Production log collection |

## --log-level Parameter

The CLI accepts the `--log-level` / `-l` parameter (default `info`):

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

Supported levels: `trace`, `debug`, `info`, `warn`, `error`.

## Log Level Usage Conventions

| Level | Purpose | Typical CLI Scenarios |
| --- | --- | --- |
| `info` | Important operations | Container create/start/stop, migration start/complete |
| `warn` | Potential issues | Migration retries, container exists but in abnormal state |
| `error` | Errors | Container crash, migration failure, network creation failure |
| `debug` | Debug information | (Currently unused, reserved for future) |
| `trace` | Detailed flow | (Currently unused, reserved for future) |

## Design Principles

1. **CLI does not swallow errors**: All errors propagate upward via `anyhow::Result`; `main()` automatically prints the error chain.
1. **Every operation start has a log**: `creating network...`, `running migrations...`, `building shittim_chest...` — the user knows what the CLI is doing.
1. **Every operation completion has confirmation**: `shittim-chest started on 0.0.0.0:80`, `all services started`.
1. **Silently-succeeding operations are not logged**: `ensure_network` does not print if the network already exists, to avoid noise.
1. **Container logs are fetched via Docker API**: The CLI itself does not write business logs, only orchestration operation logs.

## Alignment with entelecheia

| Feature | entelecheia CLI | shittim-chest CLI | Aligned |
| --- | --- | --- | --- |
| Framework | `tracing` | `tracing` | ✅ |
| Time format | `ShortTimer` (HH:MM:SS) | `ShortTimer` (HH:MM:SS) | ✅ |
| Output target | stderr | stderr | ✅ |
| Compact mode | `.compact()` | `.compact()` | ✅ |
| Hide target | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | Supported | Supported | ✅ |

The CLI log output of both projects is visually identical, making it easy for developers to switch between the two projects.
