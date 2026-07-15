#!/usr/bin/env bash
#
# celestia-install.sh — Celestia unified installer for Linux / macOS.
#
# ╔══════════════════════════════════════════════════════════════════════════════╗
# ║  ISOLATION WARNING                                                         ║
# ║  This script installs Docker Engine, systemd services, and builds Rust      ║
# ║  crates. It is STRONGLY recommended to run inside a clean Linux VM or       ║
# ║  container sandbox — NOT on your daily workstation.                         ║
# ║                                                                            ║
# ║  If running on a workstation, use a dedicated VM (multipass, lima, UTM)     ║
# ║  or a disposable LXC container. Do NOT pollute your host OS with apt/dnf    ║
# ║  packages that may conflict with your existing development environment.     ║
# ╚══════════════════════════════════════════════════════════════════════════════╝
#
# Installs all four celestia project群 components from LOCAL source code:
#   - entelecheia  (scepter server)   → built & run natively
#   - evernight    (broker)           → built natively
#   - scriptum     (TUI)              → built natively
#   - shittim-chest (GUI/CLI shell)   → built natively (Tauri or web UI)
#
# Idempotent: re-running skips work that is already done. Uses the local
# checkout at --source-root (auto-detected from this script's location by
# walking three levels up). NEVER clones from GitHub.
#
# Usage:
#   ./celestia-install.sh
#   ./celestia-install.sh --source-root /path/to/celestia
#   ./celestia-install.sh --skip-docker --skip-build
#   ./celestia-install.sh --quick --no-mirror
#
set -euo pipefail

# ── Defaults ────────────────────────────────────────────────────────────────
SOURCE_ROOT=""
SKIP_DOCKER=0
SKIP_BUILD=0
SKIP_SHORTCUTS=0
NO_MIRROR=0
MIRROR=""
DEV=0
QUICK=0

SCEPTER_PORT=8424
DOCKER_IMAGES=("pgvector/pgvector:pg18-bookworm")
STATE_FILE="${TMPDIR:-/tmp}/celestia-install.state"
INSTALL_DIR="${HOME}/.local/share/celestia"
LOG_DIR="${HOME}/.local/share/celestia/logs"
SCEPTER_PID_FILE="${LOG_DIR}/scepter.pid"
SCEPTER_LOG="${LOG_DIR}/scepter.log"

# Auto-detect OS
OS_KERNEL="$(uname -s)"
case "$OS_KERNEL" in
    Linux*)  OS_ID="linux" ;;
    Darwin*) OS_ID="macos" ;;
    *)       OS_ID="unknown" ;;
esac

# ── Helpers ─────────────────────────────────────────────────────────────────

c_info()  { printf '\033[1;34m[INFO]  %s\033[0m\n'  "$*" ; }
c_ok()    { printf '\033[1;32m[OK]    %s\033[0m\n'  "$*" ; }
c_warn()  { printf '\033[1;33m[WARN]  %s\033[0m\n'  "$*" ; }
c_err()   { printf '\033[1;31m[ERROR] %s\033[0m\n' "$*" ; }
c_step()  { printf '\n\033[1;36m==> %s\033[0m\n'   "$*" ; }

confirm_prompt() {
    # confirm_prompt "Message" [default-yes|default-no]
    local msg="$1" default="${2:-yes}"
    if [[ "$QUICK" -eq 1 ]]; then
        [[ "$default" == "yes" ]] && return 0 || return 1
    fi
    local yn
    if [[ "$default" == "yes" ]]; then yn="[Y/n]"; else yn="[y/N]"; fi
    read -r -p "$msg $yn " response </dev/tty || response=""
    [[ -z "$response" ]] && { [[ "$default" == "yes" ]] && return 0 || return 1; }
    [[ "$response" =~ ^[Yy] ]]
}

save_state() { echo "STAGE=$1" > "$STATE_FILE"; }

# Run with sudo if not root, else directly
run_root() {
    if [[ "$(id -u)" -eq 0 ]]; then "$@"
    elif sudo -n true 2>/dev/null; then sudo "$@"
    else
        c_warn "Root required for: $*"
        sudo "$@"
    fi
}

# ── Argument parsing ────────────────────────────────────────────────────────

usage() {
    cat <<'EOF'
Usage: celestia-install.sh [options]

Options:
  --source-root PATH   Override celestia source root (must contain entelecheia/Cargo.toml)
  --skip-docker        Skip Docker Engine installation
  --skip-build         Skip all cargo builds
  --skip-shortcuts     Skip desktop entry / .app bundle creation
  --no-mirror          Disable Docker registry mirror auto-configuration
  --mirror URL         Override Docker registry mirror URL
  --dev                Reserved (compat with legacy installer)
  --quick              Non-interactive; auto-accept all prompts
  -h, --help           Show this help
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --source-root)    SOURCE_ROOT="$2"; shift 2 ;;
        --skip-docker)    SKIP_DOCKER=1; shift ;;
        --skip-build)     SKIP_BUILD=1; shift ;;
        --skip-shortcuts) SKIP_SHORTCUTS=1; shift ;;
        --no-mirror)      NO_MIRROR=1; shift ;;
        --mirror)         MIRROR="$2"; shift 2 ;;
        --dev)            DEV=1; shift ;;
        --quick)          QUICK=1; shift ;;
        -h|--help)        usage; exit 0 ;;
        *) c_err "Unknown option: $1"; usage; exit 1 ;;
    esac
done

# ── Phase 1: Prerequisites ─────────────────────────────────────────────────

test_rust() {
    c_step "Phase 1: Checking prerequisites ($OS_ID)"
    if command -v rustc >/dev/null 2>&1; then
        c_ok "Rust: $(rustc --version)"
        return 0
    fi
    c_warn "rustc not found — installing via rustup..."
    if confirm_prompt "Install Rust via rustup?" yes; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        # shellcheck disable=SC1091
        source "${HOME}/.cargo/env"
        c_ok "Rust installed: $(rustc --version)"
        return 0
    fi
    c_err "Rust is required. Install from https://rustup.rs and re-run."
    return 1
}

resolve_source_root() {
    if [[ -n "$SOURCE_ROOT" && -f "$SOURCE_ROOT/entelecheia/Cargo.toml" ]]; then
        return 0
    fi
    # scripts/install/ → ../../.. = celestia source root
    local script_dir
    script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local try="$script_dir/../.."
    if [[ -f "$try/entelecheia/Cargo.toml" ]]; then
        SOURCE_ROOT="$(cd "$try" && pwd)"
        return 0
    fi
    if [[ "$QUICK" -eq 0 ]]; then
        read -r -p "Enter celestia source root path: " SOURCE_ROOT </dev/tty
        if [[ -n "$SOURCE_ROOT" && -f "$SOURCE_ROOT/entelecheia/Cargo.toml" ]]; then
            SOURCE_ROOT="$(cd "$SOURCE_ROOT" && pwd)"
            return 0
        fi
    fi
    c_err "Could not resolve celestia source root (must contain entelecheia/Cargo.toml)."
    c_err "Re-run with --source-root <path>."
    exit 1
}

# ── Phase 2: Docker setup ──────────────────────────────────────────────────

install_docker_linux() {
    c_step "Phase 2: Setting up Docker Engine"
    if command -v docker >/dev/null 2>&1; then
        c_ok "Docker present: $(docker --version)"
        if docker info >/dev/null 2>&1; then
            c_ok "Docker daemon already running"
            return 0
        fi
        c_warn "Docker daemon not running — starting..."
        run_root service docker start 2>/dev/null || \
        run_root systemctl start docker 2>/dev/null || true
        sleep 3
        if docker info >/dev/null 2>&1; then
            c_ok "Docker daemon started"
            return 0
        fi
    fi

    # Try linuxmirrors.cn one-click Docker installer first (auto-selects
    # best mirror for apt + Docker repos — critical for China networks).
    c_info "Attempting Docker install via linuxmirrors.cn (auto-mirror)..."
    local mirror_ok=false
    if curl -fsSL --connect-timeout 10 --max-time 30 \
        "https://linuxmirrors.cn/docker.sh" \
        -o /tmp/celestia-docker-install.sh 2>/dev/null; then
        if run_root bash /tmp/celestia-docker-install.sh 2>&1; then
            c_ok "Docker installed via linuxmirrors.cn mirror"
            mirror_ok=true
        fi
        rm -f /tmp/celestia-docker-install.sh
    fi

    if $mirror_ok; then
        if docker info >/dev/null 2>&1; then
            c_ok "Docker Engine installed and running"
        else
            c_warn "Docker installed but daemon may need manual start."
        fi
        return 0
    fi

    # Fallback: manual Docker repo setup.
    c_warn "linuxmirrors.cn unreachable — falling back to manual Docker install."
    local install_script
    install_script="$(cat <<'DOCKEREOF'
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq
apt-get install -y -qq ca-certificates curl gnupg lsb-release
install -m 0755 -d /etc/apt/keyrings
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.asc 2>/dev/null
chmod a+r /etc/apt/keyrings/docker.asc
echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list
apt-get update -qq
apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin fuse-overlayfs
service docker start 2>/dev/null || systemctl enable --now docker 2>/dev/null || true
usermod -aG docker "${SUDO_USER:-$USER}" 2>/dev/null || true
echo "DOCKER_INSTALLED=yes"
DOCKEREOF
)"
    if echo "$install_script" | run_root bash 2>&1 | grep -q "DOCKER_INSTALLED"; then
        c_ok "Docker installed"
    else
        c_warn "Docker install output unexpected. Verifying..."
    fi
    docker info >/dev/null 2>&1 || c_warn "Docker daemon may not be running. Try: sudo service docker start"
}

install_docker_macos() {
    c_step "Phase 2: Setting up Docker Desktop (macOS)"
    if command -v docker >/dev/null 2>&1; then
        c_ok "Docker present: $(docker --version)"
        if docker info >/dev/null 2>&1; then
            c_ok "Docker daemon already running"
            return 0
        fi
    fi
    c_warn "Docker not found. Please install Docker Desktop manually:"
    c_warn "  https://docs.docker.com/desktop/mac/install/"
    if confirm_prompt "Open Docker Desktop download page?" yes; then
        open "https://docs.docker.com/desktop/mac/install/" 2>/dev/null || true
    fi
    c_err "After installing Docker Desktop, re-run this script."
    exit 1
}

install_fuse_overlayfs() {
    c_info "Verifying fuse-overlayfs (rootless cosmos isolation)..."
    if [[ "$OS_ID" == "linux" ]]; then
        if command -v fuse-overlayfs >/dev/null 2>&1; then
            c_ok "fuse-overlayfs: $(fuse-overlayfs --version 2>&1 | head -1)"
        else
            c_warn "fuse-overlayfs missing — installing..."
            run_root apt-get install -y -qq fuse-overlayfs 2>/dev/null || \
            c_warn "Could not install fuse-overlayfs automatically (non-Debian?)."
        fi
    fi
}

configure_docker_mirror() {
    if [[ "$NO_MIRROR" -eq 1 ]]; then
        c_info "Skipping Docker mirror (--no-mirror)"
        return 0
    fi
    c_step "Configuring Docker registry mirrors"

    local mirror_script
    mirror_script="$(cat <<MIRROREOF
set -euo pipefail
NO_MIRROR=$NO_MIRROR
MIRROR_URL="$MIRROR"
IN_CHINA=false
if curl -s --connect-timeout 3 --max-time 5 https://www.baidu.com >/dev/null 2>&1; then IN_CHINA=true; fi
if [[ "\$NO_MIRROR" == "1" ]]; then echo "SKIP"; exit 0; fi
MIRRORS=()
if [[ -n "\$MIRROR_URL" ]]; then
    MIRRORS+=("\$MIRROR_URL")
elif \$IN_CHINA; then
    MIRRORS+=("https://docker.1ms.run" "https://docker.xuanyuan.me" "https://docker.m.daocloud.io")
else
    echo "NO_MIRROR_NEEDED"; exit 0
fi
MIRROR_JSON=\$(printf '%s\n' "\${MIRRORS[@]}" | jq -R . | jq -s . 2>/dev/null || \
    python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "\${MIRRORS[@]}")
mkdir -p /etc/docker
TMP=\$(mktemp)
if [[ -f /etc/docker/daemon.json ]]; then
    if command -v jq >/dev/null 2>&1; then
        jq --argjson m "\$MIRROR_JSON" '.["registry-mirrors"]=\$m' /etc/docker/daemon.json > "\$TMP"
    else
        python3 -c "import json; d=json.load(open('/etc/docker/daemon.json')); d['registry-mirrors']=\$MIRROR_JSON; json.dump(d, open('\$TMP','w'), indent=2)"
    fi
else
    echo "{\\"registry-mirrors\\": \$MIRROR_JSON}" > "\$TMP"
fi
cp "\$TMP" /etc/docker/daemon.json; rm -f "\$TMP"
service docker restart 2>/dev/null || systemctl restart docker 2>/dev/null || true
echo "MIRRORS_CONFIGURED: \${MIRRORS[*]}"
MIRROREOF
)"
    local result
    result="$(echo "$mirror_script" | run_root bash 2>&1 || true)"
    if echo "$result" | grep -q "SKIP"; then
        c_info "Mirror skipped"
    elif echo "$result" | grep -q "NO_MIRROR_NEEDED"; then
        c_info "No mirror needed (not in China)"
    elif echo "$result" | grep -q "MIRRORS_CONFIGURED"; then
        c_ok "Docker mirrors configured"
    else
        c_warn "Mirror configuration may have failed"
    fi
}

pull_docker_images() {
    c_step "Pulling Docker images"
    for img in "${DOCKER_IMAGES[@]}"; do
        c_info "Pulling: $img"
        if docker pull "$img" 2>/dev/null; then
            c_ok "Pulled: $img"
            continue
        fi
        # Retry twice
        local pulled=0
        for _ in 2 3; do
            sleep 2
            if docker pull "$img" 2>/dev/null; then
                c_ok "Pulled: $img"; pulled=1; break
            fi
        done
        if [[ "$pulled" -eq 0 ]]; then
            if docker image inspect "$img" >/dev/null 2>&1; then
                c_ok "Image already present: $img"
            else
                c_err "Failed to pull: $img"
            fi
        fi
    done
}

# ── Phase 3: Build entelecheia + evernight ─────────────────────────────────

ensure_entelecheia_env() {
    c_step "Ensuring entelecheia/.env exists"
    local dir="$SOURCE_ROOT/entelecheia"
    if [[ -f "$dir/.env" ]]; then
        c_ok ".env already exists"
    elif [[ -f "$dir/.env.example.minimal" ]]; then
        cp "$dir/.env.example.minimal" "$dir/.env"
        c_ok ".env created from .env.example.minimal"
    elif [[ -f "$dir/.env.example" ]]; then
        cp "$dir/.env.example" "$dir/.env"
        c_ok ".env created from .env.example"
    else
        cat > "$dir/.env" <<'ENVEOF'
LLM_API_KEY=sk-your-key-here
LLM_BASE_URL=https://api.openai.com/v1
LLM_MODEL=gpt-4o
DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
SERVER_BIND_ADDRESS=127.0.0.1:8424
RUST_LOG=info
ENVEOF
        c_ok ".env created (minimal)"
    fi
    c_warn "Edit LLM_API_KEY etc. in: $dir/.env"
}

build_entelecheia() {
    if [[ "$SKIP_BUILD" -eq 1 ]]; then
        c_info "Skipping entelecheia build (--skip-build)"; return 1
    fi
    c_step "Phase 3: Building entelecheia scepter (release)"
    local dir="$SOURCE_ROOT/entelecheia"
    if (cd "$dir" && cargo build --release -p scepter); then
        c_ok "scepter built (release)"
        return 0
    fi
    c_err "scepter build failed. Run: cd $dir && cargo build --release -p scepter"
    return 1
}

build_evernight() {
    if [[ "$SKIP_BUILD" -eq 1 ]]; then
        c_info "Skipping evernight build (--skip-build)"; return 1
    fi
    c_step "Building evernight (release)"
    local dir="$SOURCE_ROOT/evernight"
    if (cd "$dir" && cargo build --release -p evernight); then
        c_ok "evernight built (release)"
        return 0
    fi
    c_err "evernight build failed. Run: cd $dir && cargo build --release -p evernight"
    return 1
}

start_postgres() {
    c_step "Starting PostgreSQL via docker compose"
    local dir="$SOURCE_ROOT/entelecheia"
    local compose_file
    if [[ -f "$dir/tests/docker/docker-compose.e2e.yml" ]]; then
        compose_file="tests/docker/docker-compose.e2e.yml"
    elif [[ -f "$dir/docker-compose.yml" ]]; then
        compose_file="docker-compose.yml"
    else
        c_warn "No docker-compose file found — skip postgres start"
        return 0
    fi
    (cd "$dir" && docker compose -f "$compose_file" up -d postgres 2>&1) || \
    (cd "$dir" && docker-compose -f "$compose_file" up -d postgres 2>&1) || {
        c_err "docker compose up failed"
        return 1
    }
    c_info "Waiting for PostgreSQL..."
    local i
    for i in $(seq 1 30); do
        if docker ps --format '{{.Names}} {{.Status}}' | grep -i postgres | grep -qi healthy; then
            c_ok "PostgreSQL is ready"
            return 0
        fi
        local pg_name
        pg_name="$(docker ps --filter name=postgres --format '{{.Names}}' | head -1)"
        if [[ -n "$pg_name" ]]; then
            if docker exec "$pg_name" pg_isready -U amphoreus 2>/dev/null || \
               docker exec "$pg_name" pg_isready -U entelecheia 2>/dev/null; then
                c_ok "PostgreSQL is ready"
                return 0
            fi
        fi
        sleep 2
    done
    c_warn "PostgreSQL did not become ready in 60s"
}

# ── Phase 4: Build scriptum + shittim-chest ────────────────────────────────

build_scriptum() {
    if [[ "$SKIP_BUILD" -eq 1 ]]; then
        c_info "Skipping scriptum build (--skip-build)"; return 1
    fi
    c_step "Phase 4: Building scriptum (TUI)"
    local dir="$SOURCE_ROOT/scriptum"
    if [[ ! -f "$dir/Cargo.toml" ]]; then
        c_warn "scriptum/ not found at $dir — skipping"; return 1
    fi
    if (cd "$dir" && cargo build --release --bin scriptum); then
        c_ok "scriptum built: $dir/target/release/scriptum"
        return 0
    fi
    c_err "scriptum build failed"
    return 1
}

build_shittim_chest() {
    # Sets SHITTIM_EXE on success.
    if [[ "$SKIP_BUILD" -eq 1 ]]; then
        c_info "Skipping shittim-chest build (--skip-build)"; return 1
    fi
    c_step "Building shittim-chest"
    local dir="$SOURCE_ROOT/shittim-chest"
    if [[ ! -f "$dir/Cargo.toml" ]]; then
        c_warn "shittim-chest/ not found at $dir — skipping"; return 1
    fi

    # Try Tauri first (GUI app). `cargo tauri build` needs cargo-tauri.
    if cargo tauri --version >/dev/null 2>&1; then
        c_info "Attempting Tauri build (may take several minutes)..."
        if (cd "$dir" && cargo tauri build); then
            local bundle
            bundle="$(find "$dir/target/release/bundle" -type f \( -name "*.exe" -o -name "*.AppImage" -o -name "*.dmg" -o -name "*.app" \) 2>/dev/null | head -1)"
            if [[ -n "$bundle" ]]; then
                SHITTIM_EXE="$bundle"
                c_ok "Tauri app built: $bundle"
                return 0
            fi
        fi
        c_warn "Tauri build did not produce a bundle. Falling back to web UI / CLI."
    else
        c_info "cargo-tauri not installed. Trying web UI + CLI build."
    fi

    # Build the web UI (pnpm --filter @celestia-island/webui build)
    if command -v pnpm >/dev/null 2>&1; then
        c_info "Building web UI..."
        (cd "$dir" && pnpm --filter @celestia-island/webui build) || \
            c_warn "Web UI build returned non-zero."
    else
        c_warn "pnpm not found — skipping web UI build."
    fi

    # Build the CLI binary (default member of the workspace).
    c_info "Building shittim-chest CLI (chest-cli)..."
    if (cd "$dir" && cargo build --release -p shittim-chest-cli); then
        local exe
        if [[ "$OS_ID" == "macos" ]]; then
            exe="$dir/target/release/chest-cli"
        else
            exe="$dir/target/release/chest-cli"
        fi
        if [[ -f "$exe" ]]; then
            SHITTIM_EXE="$exe"
            c_ok "shittim-chest CLI built: $exe"
            return 0
        fi
    fi
    c_err "shittim-chest build failed."
    return 1
}

# ── Phase 5: Desktop entries / .app bundles ───────────────────────────────

create_desktop_entry_linux() {
    # create_desktop_entry_linux NAME EXEC ICON COMMENT TERMINAL
    local name="$1" exec_cmd="$2" icon="$3" comment="$4" terminal="$5"
    local apps_dir="${HOME}/.local/share/applications"
    local icons_dir="${HOME}/.local/share/icons/hicolor/256x256/apps"
    mkdir -p "$apps_dir" "$icons_dir" "$INSTALL_DIR"
    local file="$apps_dir/${name// /-}.desktop"
    cat > "$file" <<EOF
[Desktop Entry]
Type=Application
Name=$name
Exec=$exec_cmd
Icon=${icon:-celestia}
Comment=$comment
Terminal=$terminal
Categories=Development;Utility;
EOF
    chmod +x "$file"
    c_ok "Desktop entry created: $file"
    # Refresh desktop database (best-effort)
    update-desktop-database "$apps_dir" 2>/dev/null || true
}

create_app_bundle_macos() {
    # create_app_bundle_macos NAME EXEC_PATH COMMENT
    local name="$1" exec_path="$2" comment="$3"
    local apps_dir="${HOME}/Applications"
    mkdir -p "$apps_dir" "$INSTALL_DIR"
    local safe_name="${name// /}"
    local bundle="$apps_dir/$safe_name.app"
    rm -rf "$bundle"
    mkdir -p "$bundle/Contents/MacOS"
    # Wrapper script that exec's the binary
    cat > "$bundle/Contents/MacOS/run.sh" <<EOF
#!/usr/bin/env bash
# $comment
exec "$exec_path" "\$@"
EOF
    chmod +x "$bundle/Contents/MacOS/run.sh"
    cat > "$bundle/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleExecutable</key>
    <string>run.sh</string>
    <key>CFBundleIdentifier</key>
    <string>island.celestia.$safe_name</string>
    <key>CFBundleName</key>
    <string>$name</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>1.0.0</string>
    <key>CFBundleVersion</key>
    <string>1</string>
</dict>
</plist>
EOF
    c_ok ".app bundle created: $bundle"
}

install_shortcuts() {
    if [[ "$SKIP_SHORTCUTS" -eq 1 ]]; then
        c_info "Skipping shortcut creation (--skip-shortcuts)"; return 0
    fi
    c_step "Phase 5: Installing shortcuts ($OS_ID)"
    mkdir -p "$INSTALL_DIR"

    # Scriptum (terminal app)
    local scriptum_src="$SOURCE_ROOT/scriptum/target/release/scriptum"
    if [[ -f "$scriptum_src" ]]; then
        local scriptum_dest="$INSTALL_DIR/scriptum"
        cp -f "$scriptum_src" "$scriptum_dest"
        chmod +x "$scriptum_dest"
        if [[ "$OS_ID" == "linux" ]]; then
            create_desktop_entry_linux "Scriptum" \
                "x-terminal-emulator -e $scriptum_dest" \
                "" "Celestia Scriptum TUI" "true"
        elif [[ "$OS_ID" == "macos" ]]; then
            create_app_bundle_macos "Scriptum" "$scriptum_dest" "Celestia Scriptum TUI"
        fi
    else
        c_warn "scriptum binary not available — no shortcut created."
    fi

    # Shittim-chest (GUI app or CLI launcher)
    if [[ -n "${SHITTIM_EXE:-}" && -f "$SHITTIM_EXE" ]]; then
        local dest_name
        dest_name="$(basename "$SHITTIM_EXE")"
        local shittim_dest="$INSTALL_DIR/$dest_name"
        cp -f "$SHITTIM_EXE" "$shittim_dest"
        chmod +x "$shittim_dest"
        local is_gui=1
        case "$dest_name" in
            chest-cli*) is_gui=0 ;;
        esac
        if [[ "$OS_ID" == "linux" ]]; then
            if [[ "$is_gui" -eq 1 ]]; then
                create_desktop_entry_linux "Shittim Chest" \
                    "$shittim_dest" "" "Celestia Shittim-chest desktop app" "false"
            else
                create_desktop_entry_linux "Shittim Chest" \
                    "x-terminal-emulator -e $shittim_dest" \
                    "" "Celestia Shittim-chest (CLI)" "true"
            fi
        elif [[ "$OS_ID" == "macos" ]]; then
            create_app_bundle_macos "Shittim Chest" "$shittim_dest" \
                "Celestia Shittim-chest desktop app"
        fi
    else
        c_warn "shittim-chest binary not available — no shortcut created."
    fi
}

# ── Phase 6: Start services + summary ──────────────────────────────────────

start_scepter() {
    c_step "Phase 6: Starting scepter server"
    if [[ "$1" != "ok" ]]; then
        c_warn "scepter was not built — skipping auto-start."
        c_warn "Start manually: $SOURCE_ROOT/entelecheia/target/release/scepter"
        return 0
    fi
    local dir="$SOURCE_ROOT/entelecheia"
    mkdir -p "$LOG_DIR"
    if [[ -f "$SCEPTER_PID_FILE" ]] && kill -0 "$(cat "$SCEPTER_PID_FILE")" 2>/dev/null; then
        c_ok "scepter already running (pid: $(cat "$SCEPTER_PID_FILE"))"
        return 0
    fi
    # cd then background scepter directly so $! is scepter's PID (not a subshell's).
    cd "$dir" || { c_err "Cannot cd to $dir"; return 1; }
    nohup ./target/release/scepter >"$SCEPTER_LOG" 2>&1 &
    local pid=$!
    # Return to the original directory.
    cd - >/dev/null || true
    echo "$pid" > "$SCEPTER_PID_FILE"
    sleep 2
    if kill -0 "$pid" 2>/dev/null; then
        c_ok "scepter started (pid: $pid)"
    else
        c_err "scepter failed to start — check $SCEPTER_LOG"
    fi
}

show_summary() {
    # show_summary SCEPTER_OK EVERNIGHT_OK SCRIPTUM_OK SHITTIM_OK
    local sep
    sep="$(printf '─%.0s' {1..64})"
    echo ""
    echo "$sep"
    echo "  Celestia Installation Complete"
    echo "$sep"
    echo ""
    echo "  Components built:"
    for entry in "scepter (entelecheia)|$1" "evernight|$2" "scriptum (TUI)|$3" "shittim-chest|$4"; do
        local name="${entry%%|*}"
        local ok="${entry##*|}"
        local mark
        [[ "$ok" == "ok" ]] && mark="[OK]" || mark="[MISSING]"
        printf "    %-22s %s\n" "$name" "$mark"
    done
    echo ""
    echo "  Paths:"
    echo "    Source root:       $SOURCE_ROOT"
    echo "    entelecheia/.env:  $SOURCE_ROOT/entelecheia/.env"
    echo "    scepter log:       $SCEPTER_LOG"
    echo "    scepter pid:       $SCEPTER_PID_FILE"
    echo "    Install dir:       $INSTALL_DIR"
    if [[ "$OS_ID" == "linux" ]]; then
        echo "    Desktop entries:   ${HOME}/.local/share/applications/"
    elif [[ "$OS_ID" == "macos" ]]; then
        echo "    App bundles:       ${HOME}/Applications/"
    fi
    echo ""
    echo "  Services:"
    echo "    scepter (HTTP/WS): http://localhost:$SCEPTER_PORT"
    echo "    PostgreSQL:        localhost:5432"
    echo ""
    echo "  How to use:"
    echo "    • Launch Scriptum from your application launcher → TUI connects to scepter."
    echo "    • Launch Shittim Chest from your application launcher → desktop app / CLI."
    echo "    • Edit .env:        \$EDITOR $SOURCE_ROOT/entelecheia/.env"
    echo "    • Tail scepter log: tail -f $SCEPTER_LOG"
    echo ""
    echo "  Stop / restart:"
    echo "    Stop scepter:  kill \$(cat $SCEPTER_PID_FILE)"
    local compose_file="tests/docker/docker-compose.e2e.yml"
    [[ -f "$SOURCE_ROOT/entelecheia/$compose_file" ]] || compose_file="docker-compose.yml"
    echo "    Stop postgres: (cd $SOURCE_ROOT/entelecheia && docker compose -f $compose_file down)"
    echo "    Restart all:   re-run this script (idempotent)."
    echo ""
    echo "$sep"
}

# ── Main ───────────────────────────────────────────────────────────────────

main() {
    echo ""
    printf '%.0s─' {1..64}; echo
    echo "  Celestia Unified Installer ($OS_ID native)"
    printf '%.0s─' {1..64}; echo
    c_info "OS: $(uname -sr)"

    # Phase 1
    if ! test_rust; then
        exit 1
    fi
    resolve_source_root
    c_ok "Celestia source root: $SOURCE_ROOT"

    # Phase 2
    if [[ "$SKIP_DOCKER" -eq 0 ]]; then
        if [[ "$OS_ID" == "macos" ]]; then
            install_docker_macos
        else
            install_docker_linux
        fi
        configure_docker_mirror
        install_fuse_overlayfs
        pull_docker_images
    else
        c_info "Skipping Docker setup (--skip-docker)"
    fi

    # Phase 3
    ensure_entelecheia_env
    local scepter_ok="fail" evernight_ok="fail"
    if build_entelecheia; then scepter_ok="ok"; fi
    if build_evernight;  then evernight_ok="ok"; fi
    start_postgres

    # Phase 4
    local scriptum_ok="fail" shittim_ok="fail"
    SHITTIM_EXE=""
    if build_scriptum; then scriptum_ok="ok"; fi
    if build_shittim_chest; then shittim_ok="ok"; fi

    # Phase 5
    install_shortcuts

    # Phase 6
    start_scepter "$scepter_ok"
    save_state "complete"
    show_summary "$scepter_ok" "$evernight_ok" "$scriptum_ok" "$shittim_ok"
}

# Reboot-resume support: clear stale state from a previous Docker install reboot.
if [[ -f "$STATE_FILE" ]]; then
    stage="$(grep -E '^STAGE=' "$STATE_FILE" | cut -d= -f2 || true)"
    if [[ "$stage" == "reboot-pending" ]]; then
        c_info "Resuming after reboot..."
        rm -f "$STATE_FILE"
    fi
fi

# Run main with error handling
if ! main "$@"; then
    c_err "Installer failed."
    exit 1
fi
