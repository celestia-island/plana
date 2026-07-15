# Celestia Unified Installer

> arona/scripts/install/ — the **single source of truth** for celestia installation.
> Do NOT maintain separate install scripts in entelecheia/evernight/scriptum/shittim-chest.

## Scripts

| Script | Platform | Purpose |
|--------|----------|---------|
| `celestia-install.ps1` | Windows (PowerShell 5.1+) | Full install: creates isolated `celestia-XXX` WSL2 instance, builds all 4 projects, adds Start Menu shortcuts |
| `celestia-install.sh` | Linux / macOS (bash) | Full install: Docker + Rust + builds all 4 projects + desktop entries / .app bundles |
| `celestia-init.sh` | Inside WSL2 instance (bash, root) | WSL2 first-boot init: apt mirrors, Docker Engine, fuse-overlayfs, pull pgvector image, prepare workspace |

## Quick start

### Windows

```powershell
# From the celestia source root:
.\arona\scripts\install\celestia-install.ps1

# Skip builds (already built):
.\arona\scripts\install\celestia-install.ps1 -SkipBuild -Quick

# Custom source root:
.\arona\scripts\install\celestia-install.ps1 -SourceRoot D:\src\celestia
```

### Linux / macOS

```bash
# From the celestia source root:
bash arona/scripts/install/celestia-install.sh

# Skip Docker setup:
bash arona/scripts/install/celestia-install.sh --skip-docker --quick
```

## Isolation principle (Windows)

The Windows installer follows the **Docker Desktop pattern**:

- It creates a **dedicated** `celestia-XXX` WSL2 instance (3-digit random suffix)
- It **never** touches your `Ubuntu-24.04` or any other pre-existing WSL distro
- All `apt-get`, `systemd`, `docker` operations execute **exclusively** inside `celestia-XXX`
- The instance name is persisted to `%CELESTIA_WSL_INSTANCE%` (User env var)
- Multiple `celestia-XXX` instances can co-exist (e.g., `celestia-007` for v1.0, `celestia-128` for v1.1)

To clean up:

```powershell
# List celestia instances:
wsl -l -q | Select-String 'celestia-\d{3}'

# Remove a specific instance:
wsl --unregister celestia-007
```

## Flags

### celestia-install.ps1

| Flag | Effect |
|------|--------|
| `-SourceRoot <path>` | Override celestia source root |
| `-SkipDocker` | Skip Docker setup in WSL |
| `-SkipBuild` | Skip all `cargo build` steps |
| `-SkipShortcuts` | Skip Start Menu shortcut creation |
| `-NoMirror` | Disable Docker registry mirror auto-config |
| `-Mirror <url>` | Override Docker mirror URL |
| `-Quick` | Non-interactive, auto-accept all prompts |

### celestia-install.sh

| Flag | Effect |
|------|--------|
| `--source-root <path>` | Override celestia source root |
| `--skip-docker` | Skip Docker Engine installation |
| `--skip-build` | Skip all cargo builds |
| `--skip-shortcuts` | Skip desktop entry / .app bundle creation |
| `--no-mirror` | Disable Docker registry mirror auto-config |
| `--mirror <url>` | Override Docker mirror URL |
| `--quick` / `-q` | Non-interactive mode |

## What it installs

1. **Docker Engine** — container runtime (with China mirror auto-detection)
2. **fuse-overlayfs** — rootless container storage for cosmos isolation
3. **pgvector/pgvector:pg18-bookworm** — PostgreSQL with vector extension
4. **entelecheia scepter** — multi-agent orchestration server (Rust, release)
5. **evernight** — industrial I/O broker (Rust, release)
6. **scriptum** — developer TUI (Rust, release)
7. **shittim-chest** — desktop app / web UI (Tauri + pnpm)

## Post-install

- **scepter** starts automatically and runs on `http://localhost:8424`
- **scriptum** and **shittim-chest** shortcuts appear in Start Menu (Windows) or app launcher (Linux/macOS)
- Edit `entelecheia/.env` to configure your LLM API key

## Legacy scripts

The old per-project installers are **deprecated** but retained for backward compatibility:

- `entelecheia/scripts/deploy/install.ps1` → use `arona/scripts/install/celestia-install.ps1`
- `entelecheia/scripts/deploy/install.sh` → use `arona/scripts/install/celestia-install.sh`
