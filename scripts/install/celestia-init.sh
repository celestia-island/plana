#!/bin/sh
#
# celestia-init.sh — First-boot init for celestia-XXX WSL2 instance.
#
# Runs inside a freshly-imported Alpine Linux WSL2 instance. No apt, no
# systemd, no host-side effects. Mirror selection via linuxmirrors.cn-style
# auto-detection (fastest available, not hardcoded).
#
# What it does:
#   1. Detect fastest Alpine + pip mirror (touch linuxmirrors.cn style)
#   2. Install podman + curl + bash + python3
#   3. Configure Docker registry mirrors (auto-detect best)
#   4. Write instance.toml for endpoint discovery
#
# Usage (run from INSIDE the celestia-XXX WSL2 instance):
#   CELESTIA_INSTANCE_ID=42 sh celestia-init.sh
#

set -eu

c_info()  { printf '\033[1;34m[INIT]  %s\033[0m\n'  "$*"; }
c_ok()    { printf '\033[1;32m[INIT]  %s\033[0m\n'  "$*"; }
c_warn()  { printf '\033[1;33m[INIT]  %s\033[0m\n'  "$*"; }
c_step()  { printf '\n\033[1;36m[INIT]  ==> %s\033[0m\n'  "$*"; }

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
apk add podman podman-docker curl bash python3 py3-pip shadow fuse-overlayfs
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

# ── Step 4: Configure pip mirror ─────────────────────────────────────────────

c_step "Step 4: Configuring pip mirror"
if detect_china; then
    PIP_INDEX=$(fastest_mirror \
        "https://mirrors.aliyun.com/pypi/simple/" \
        "https://mirrors.tuna.tsinghua.edu.cn/pypi/web/simple/" \
        "https://pypi.org/simple/")
    c_ok "Fastest pip: $PIP_INDEX"
    mkdir -p ~/.config/pip
    cat > ~/.config/pip/pip.conf <<EOF
[global]
index-url = ${PIP_INDEX}
EOF
fi

# ── Step 5: Write instance.toml ──────────────────────────────────────────────

c_step "Step 5: Writing instance discovery endpoint"
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

# ── Done ─────────────────────────────────────────────────────────────────────

echo ""
printf '%.0s─' {1..60}; echo
echo "  celestia-init complete — ${INSTANCE_NAME}"
printf '%.0s─' {1..60}; echo
echo "  podman:    $(podman --version 2>/dev/null || echo MISSING)"
echo "  python:    $(python3 --version 2>/dev/null || echo MISSING)"
echo "  alpine:    $(cat /etc/alpine-release 2>/dev/null || echo unknown)"
echo "  instance:  ${INSTANCE_NAME} (scepter port ${SCEPTER_PORT})"
echo "  mirrors:   alpine=${ALPINE_REPO} docker=${MIRROR_HOST:-none} pip=${PIP_INDEX:-default}"
printf '%.0s─' {1..60}; echo
