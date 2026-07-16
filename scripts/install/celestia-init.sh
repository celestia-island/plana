#!/bin/sh
#
# celestia-init.sh — First-boot init for celestia-XXX WSL2 instance.
#
# Runs inside a freshly-imported Alpine Linux WSL2 instance. Installs
# podman + minimal toolchain. No apt, no systemd, no host-side effects.
#
# What it does:
#   1. Install podman + curl + bash + python3
#   2. Configure Docker registry mirrors (auto-detect China)
#   3. Install celestia-devtools (pip)
#   4. Write instance.toml for endpoint discovery
#
# Usage (run from INSIDE the celestia-XXX WSL2 instance):
#   sh celestia-init.sh
#   sh celestia-init.sh --no-mirror
#   sh celestia-init.sh --mirror https://docker.1ms.run
#
set -eu

NO_MIRROR=0
MIRROR=""

while [ $# -gt 0 ]; do
    case "$1" in
        --no-mirror) NO_MIRROR=1; shift ;;
        --mirror)    MIRROR="$2"; shift 2 ;;
        -h|--help)   echo "Usage: celestia-init.sh [--no-mirror] [--mirror URL]"; exit 0 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

c_info()  { printf '\033[1;34m[INIT]  %s\033[0m\n'  "$*"; }
c_ok()    { printf '\033[1;32m[INIT]  %s\033[0m\n'  "$*"; }
c_warn()  { printf '\033[1;33m[INIT]  %s\033[0m\n'  "$*"; }
c_step()  { printf '\n\033[1;36m[INIT]  ==> %s\033[0m\n'  "$*"; }

detect_china() {
    curl -s --connect-timeout 3 --max-time 5 https://www.baidu.com >/dev/null 2>&1
}

# ── Step 1: Install podman ──────────────────────────────────────────
c_step "Step 1: Installing podman + tools"

apk update
apk add podman podman-docker curl bash python3 py3-pip shadow fuse-overlayfs
c_ok "podman $(podman --version)"

# Start cgroups (required for podman rootless)
mount -t cgroup2 cgroup2 /sys/fs/cgroup 2>/dev/null || true

# ── Step 2: Configure registry mirrors ──────────────────────────────
if [ "$NO_MIRROR" -eq 1 ]; then
    c_info "Step 2: Skipping mirror (--no-mirror)"
else
    c_step "Step 2: Configuring Docker registry mirrors"
    if [ -n "$MIRROR" ]; then
        mirrors="$MIRROR"
    elif detect_china; then
        c_info "China network detected"
        mirrors='docker.1ms.run docker.xuanyuan.me docker.m.daocloud.io'
    else
        c_ok "Not in China — no mirror needed."
        mirrors=""
    fi

    if [ -n "$mirrors" ]; then
        mkdir -p /etc/containers
        # Build registries.conf with mirror entries
        {
            echo '[[registry]]'
            echo 'prefix = "docker.io"'
            echo 'location = "docker.io"'
            for m in $mirrors; do
                echo ''
                echo '[[registry.mirror]]'
                echo "location = \"$m\""
            done
        } > /etc/containers/registries.conf
        c_ok "Mirrors: $mirrors"
    fi
fi

# ── Step 3: Install celestia-devtools ────────────────────────────────
c_step "Step 3: Installing celestia-devtools"
pip3 install --break-system-packages git+https://github.com/celestia-island/celestia-devtools.git 2>/dev/null || \
    pip3 install --break-system-packages celestia-devtools 2>/dev/null || \
    c_warn "celestia-devtools not available (will use git deps directly)"
c_ok "devtools ready"

# ── Step 4: Write instance.toml ──────────────────────────────────────
c_step "Step 4: Writing instance discovery endpoint"

instance_id="${CELESTIA_INSTANCE_ID:-$((RANDOM % 1000))}"
instance_name=$(printf "celestia-%03d" "$instance_id")
scepter_port=$((8424 + instance_id * 100))

mkdir -p ~/.config/celestia
cat > ~/.config/celestia/instance.toml <<TOML
[instance]
id = ${instance_id}
name = "${instance_name}"

[scepter]
host = "localhost"
port = ${scepter_port}
health_url = "http://localhost:${scepter_port}/health"

[projects]
root = "/celestia"
mounted = ["entelecheia", "evernight", "shittim-chest", "arona", "noa", "scriptum"]
TOML

c_ok "Wrote ~/.config/celestia/instance.toml"
c_ok "  Instance: ${instance_name}  (port=${scepter_port})"

# ── Step 5: Pull base image (optional warmup) ────────────────────────
c_step "Step 5: Pre-pulling rust builder image"
podman pull docker.io/library/rust:1.85-bookworm 2>/dev/null && \
    c_ok "rust:1.85-bookworm ready" || \
    c_warn "Pull failed (network?) — will pull on first build"

# ── Done ─────────────────────────────────────────────────────────────
echo ""
printf '%.0s─' {1..60}; echo
echo "  celestia-init complete"
printf '%.0s─' {1..60}; echo
echo "  podman:  $(podman --version 2>/dev/null || echo MISSING)"
echo "  python:  $(python3 --version 2>/dev/null || echo MISSING)"
echo "  mirrors: ${mirrors:-none}"
echo "  instance: ${instance_name} (port=${scepter_port})"
printf '%.0s─' {1..60}; echo
