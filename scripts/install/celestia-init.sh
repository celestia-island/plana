#!/bin/sh
#
# celestia-init.sh — First-boot init for celestia nodes (WSL2 Alpine / Linux).
#
# Runs either inside a freshly-imported Alpine Linux WSL2 instance, or on a
# standard Linux node (Ubuntu/Debian) for installing and starting scepter
# from pre-built binaries.
#
# No apt, no systemd, no host-side effects in WSL2 Alpine mode.
# Mirror selection via linuxmirrors.cn-style auto-detection (fastest available).
#
# What it does:
#   1. Detect OS type (Alpine WSL2 vs Debian/Ubuntu Linux)
#   2. Install podman/Docker + curl + bash + fuse-overlayfs
#   3. Configure registry mirrors (auto-detect best)
#   4. Write instance.toml + systemd service (Linux mode)
#   5. Install pre-built scepter binary from offline dir
#
# Usage (standalone on a Linux node):
#   CELESTIA_INSTANCE_ID=42 sh celestia-init.sh
#   CELESTIA_INSTANCE_ID=42 sh celestia-init.sh --offline-dir /tmp/offline
#
# Usage (from celestia-install.sh for remote target-init):
#   sh celestia-init.sh --offline-dir /tmp/offline --target-ip 192.168.2.148 --target-pass hydroSinap2024
#

set -eu

c_info()  { printf '\033[1;34m[INIT]  %s\033[0m\n'  "$*"; }
c_ok()    { printf '\033[1;32m[INIT]  %s\033[0m\n'  "$*"; }
c_warn()  { printf '\033[1;33m[INIT]  %s\033[0m\n'  "$*"; }
c_err()   { printf '\033[1;31m[INIT]  %s\033[0m\n'  "$*"; }
c_step()  { printf '\n\033[1;36m[INIT]  ==> %s\033[0m\n'  "$*"; }

# ── Defaults ──────────────────────────────────────────────────────────────────
OFFLINE_DIR=""
TARGET_IP=""
DEPLOY_USER="lab"
SSH_PASS="hydroSinap2024"
SCEPTER_PORT=8424
INSTALL_DIR="${HOME}/.local/share/celestia"
LOG_DIR="${HOME}/.local/share/celestia/logs"
SKIP_SERVICE=0

# ── Argument parsing ──────────────────────────────────────────────────────────
while [ $# -gt 0 ]; do
    case "$1" in
        --offline-dir)  OFFLINE_DIR="$2"; shift 2 ;;
        --target-ip)    TARGET_IP="$2"; shift 2 ;;
        --target-user)  DEPLOY_USER="$2"; shift 2 ;;
        --target-pass)  SSH_PASS="$2"; shift 2 ;;
        --skip-service) SKIP_SERVICE=1; shift ;;
        *) c_warn "Unknown option: $1"; shift ;;
    esac
done

# ── Detect OS ─────────────────────────────────────────────────────────────────
detect_os() {
    if [ -f /etc/alpine-release ]; then
        echo "alpine"
    elif [ -f /etc/os-release ]; then
        . /etc/os-release
        case "$ID" in
            debian|ubuntu|linuxmint|pop|raspbian) echo "debian" ;;
            fedora|rhel|centos|rocky|almalinux) echo "rhel" ;;
            arch|manjaro) echo "arch" ;;
            *) echo "linux" ;;
        esac
    elif [ "$(uname -s)" = "Linux" ]; then
        echo "linux"
    else
        echo "unknown"
    fi
}
OS_TYPE=$(detect_os)

# ── Resolve instance ID ──────────────────────────────────────────────────────

resolve_instance_id() {
    if [ -n "${CELESTIA_INSTANCE_ID:-}" ]; then
        echo "$CELESTIA_INSTANCE_ID"
        return
    fi
    # Parse from WSL distro name "celestia-NNN" → NNN
    local hn
    hn=$(hostname 2>/dev/null || cat /etc/hostname 2>/dev/null || echo "")
    if echo "$hn" | grep -qE '^celestia-[0-9]{3}$'; then
        echo "$hn" | sed 's/celestia-//'
        return
    fi
    # Parse from /etc/hostname in WSL (the distro name is written there by wsl --import)
    if [ -f /etc/hostname ]; then
        local name
        name=$(cat /etc/hostname)
        if echo "$name" | grep -qE '^celestia-[0-9]{3}$'; then
            echo "$name" | sed 's/celestia-//'
            return
        fi
    fi
    # Fallback
    echo $((RANDOM % 1000))
}

INSTANCE_ID=$(resolve_instance_id)
INSTANCE_NAME=$(printf "celestia-%03d" "$INSTANCE_ID")
SCEPTER_PORT=$((8424 + INSTANCE_ID * 100))

# ── Mirror detection ─────────────────────────────────────────────────────────

detect_china() {
    curl -s --connect-timeout 3 --max-time 5 https://www.baidu.com >/dev/null 2>&1
}

# Test which of a list of URLs is fastest (by connect time). Returns the winner.
fastest_mirror() {
    local best="" best_time=999
    for url in "$@"; do
        local t
        t=$(curl -s -o /dev/null -w '%{time_connect}' --connect-timeout 3 "$url" 2>/dev/null || echo "999")
        if echo "$t $best_time" | awk '{exit !($1 < $2)}'; then
            best="$url"
            best_time="$t"
        fi
    done
    echo "${best:-$1}"
}

# ── Step 1: Configure fastest Alpine mirror ───────────────────────────────────

c_step "Step 1: Configuring fastest Alpine mirror"

ALPINE_REPO="dl-cdn.alpinelinux.org"
if detect_china; then
    c_info "China network — testing Alpine mirrors..."
    ALPINE_REPO=$(fastest_mirror \
        "mirrors.tuna.tsinghua.edu.cn" \
        "mirrors.ustc.edu.cn" \
        "mirrors.aliyun.com" \
        "mirrors.163.com" \
        "dl-cdn.alpinelinux.org")
    c_ok "Fastest: $ALPINE_REPO"
    # Write APK repositories with the selected mirror
    cat > /etc/apk/repositories <<EOF
https://${ALPINE_REPO}/alpine/v3.21/main
https://${ALPINE_REPO}/alpine/v3.21/community
EOF
else
    c_ok "Using default Alpine CDN"
fi

# ── Step 2: Install tools ────────────────────────────────────────────────────

c_step "Step 2: Installing podman + tools"
apk update
apk add podman podman-docker curl bash shadow fuse-overlayfs
c_ok "podman $(podman --version)"
mount -t cgroup2 cgroup2 /sys/fs/cgroup 2>/dev/null || true

# Start podman daemon so subsequent steps (mirror test, instance.toml) work.
# In WSL2 there's no systemd/OpenRC, so we start it as a background service.
mkdir -p /var/run/podman
podman system service --time=0 unix:///var/run/podman/podman.sock &
sleep 2
c_ok "podman daemon started"

# ── Step 3: Configure Docker registry mirrors ─────────────────────────────────

c_step "Step 3: Configuring Docker registry mirrors"

mkdir -p /etc/containers
cat > /etc/containers/registries.conf <<'HEADER'
[[registry]]
prefix = "docker.io"
location = "docker.io"
HEADER

if detect_china; then
    c_info "Testing Docker mirrors..."
    DOCKER_MIRROR=$(fastest_mirror \
        "https://docker.1ms.run" \
        "https://docker.xuanyuan.me" \
        "https://docker.m.daocloud.io")
    c_ok "Fastest Docker mirror: $DOCKER_MIRROR"
    # Strip https:// prefix for registries.conf
    MIRROR_HOST=$(echo "$DOCKER_MIRROR" | sed 's|https://||')
    cat >> /etc/containers/registries.conf <<EOF

[[registry.mirror]]
location = "${MIRROR_HOST}"
EOF
else
    c_ok "Not in China — no mirror needed"
fi

# ── Step 4: Write instance.toml ──────────────────────────────────────────────

c_step "Step 4: Writing instance discovery endpoint"
mkdir -p ~/.config/celestia
cat > ~/.config/celestia/instance.toml <<TOML
[instance]
id = ${INSTANCE_ID}
name = "${INSTANCE_NAME}"

[scepter]
host = "localhost"
port = ${SCEPTER_PORT}
health_url = "http://localhost:${SCEPTER_PORT}/health"

[projects]
root = "/celestia"
mounted = ["entelecheia", "evernight", "shittim-chest", "arona", "noa", "scriptum"]
TOML
c_ok "Wrote ~/.config/celestia/instance.toml"
c_ok "  Instance: ${INSTANCE_NAME}  (port=${SCEPTER_PORT})"

# ── Linux-native init (non-Alpine) ───────────────────────────────────────────
if [ "$OS_TYPE" != "alpine" ]; then
    c_step "Linux native mode: installing scepter from offline bundle"

    mkdir -p "$INSTALL_DIR" "$LOG_DIR"

    if [ -n "$OFFLINE_DIR" ] && [ -d "$OFFLINE_DIR" ]; then
        for bin in scepter entelecheia evernight scriptum chest-cli; do
            local_src="$OFFLINE_DIR/$bin"
            if [ -f "$local_src" ]; then
                cp -f "$local_src" "$INSTALL_DIR/$bin"
                chmod +x "$INSTALL_DIR/$bin"
                c_ok "Installed $bin from offline bundle"
            fi
        done
    fi

    if [ -f "$INSTALL_DIR/scepter" ] || [ -f "$INSTALL_DIR/entelecheia" ]; then
        BINARY="$INSTALL_DIR/scepter"
        [ -f "$BINARY" ] || BINARY="$INSTALL_DIR/entelecheia"
        chmod +x "$BINARY"
        c_ok "scepter binary found at $BINARY"
    else
        c_warn "No scepter binary found in $INSTALL_DIR — deploy it first"
    fi

    c_info "Writing scepter environment config..."
    INSTALL_DIR="$INSTALL_DIR" SCEPTER_PORT="$SCEPTER_PORT" LOG_DIR="$LOG_DIR" c_info "Writing instance.toml"
fi

# If skip-service is set or on Alpine, don't create systemd services
if [ "$SKIP_SERVICE" -eq 1 ] || [ "$OS_TYPE" = "alpine" ]; then
    c_info "Skipping systemd service creation"
else
    c_step "Creating systemd service for scepter"

    SCEPTER_BIN="${INSTALL_DIR}/scepter"
    if [ ! -f "$SCEPTER_BIN" ]; then
        SCEPTER_BIN="${INSTALL_DIR}/entelecheia"
    fi

    if [ -f "$SCEPTER_BIN" ]; then
        sudo tee /etc/systemd/system/scepter.service >/dev/null <<SVC
[Unit]
Description=Entelecheia Scepter Server
After=network.target

[Service]
Type=simple
User=$USER
WorkingDirectory=${INSTALL_DIR}
ExecStart=${SCEPTER_BIN}
Restart=always
RestartSec=5
StandardOutput=append:${LOG_DIR}/scepter.log
StandardError=append:${LOG_DIR}/scepter.log

[Install]
WantedBy=multi-user.target
SVC
        sudo tee "${INSTALL_DIR}/.env" >/dev/null <<ENVEOF
SERVER_BIND_ADDRESS=0.0.0.0:${SCEPTER_PORT}
RUST_LOG=info
LLM_API_KEY=sk-your-key-here
LLM_BASE_URL=https://api.deepseek.com/v1
LLM_MODEL=deepseek-chat
ENTELECHEIA_WORKSPACE_DIR=/mnt/codespace
ENVEOF
        sudo systemctl daemon-reload
        sudo systemctl enable scepter
        sudo systemctl start scepter 2>/dev/null || c_warn "Could not start scepter — check logs"
        sleep 2
        if systemctl is-active scepter >/dev/null 2>&1; then
            c_ok "scepter service is running"
        else
            c_warn "scepter service not running — check: systemctl status scepter"
        fi
    else
        c_warn "No scepter binary — skipping systemd service"
    fi
fi

# ── Done ─────────────────────────────────────────────────────────────────────

echo ""
printf '%.0s─' {1..60}; echo
echo "  celestia-init complete — ${INSTANCE_NAME}"
printf '%.0s─' {1..60}; echo
echo "  os:        ${OS_TYPE}"
if [ "$OS_TYPE" = "alpine" ]; then
    echo "  podman:    $(podman --version 2>/dev/null || echo MISSING)"
    echo "  alpine:    $(cat /etc/alpine-release 2>/dev/null || echo unknown)"
    echo "  mirrors:   alpine=${ALPINE_REPO} docker=${MIRROR_HOST:-none}"
else
    echo "  scepter:   ${INSTALL_DIR}/scepter"
    echo "  port:      ${SCEPTER_PORT}"
    echo "  log:       ${LOG_DIR}/scepter.log"
    echo "  workspace: /mnt/codespace"
fi
echo "  instance:  ${INSTANCE_NAME} (scepter port ${SCEPTER_PORT})"
printf '%.0s─' {1..60}; echo
