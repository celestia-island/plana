#!/usr/bin/env bash
#
# celestia-init.sh — First-boot init for celestia-XXX WSL2 instance.
#
# This script is designed to run INSIDE a freshly-created celestia-XXX WSL2
# instance. It is invoked by the Windows installer after `wsl --import`.
#
# What it does:
#   1. Detect if running in China → configure apt mirrors
#   2. Install Docker Engine + fuse-overlayfs
#   3. Configure Docker registry mirrors (auto, or --mirror URL)
#   4. Pull required Docker images (pgvector)
#   5. Prepare celestia workspace directories
#   6. Output a manifest of what was installed
#
# Usage (run from INSIDE the celestia-XXX WSL2 instance):
#   sudo bash celestia-init.sh
#   sudo bash celestia-init.sh --no-mirror
#   sudo bash celestia-init.sh --mirror https://docker.1ms.run
#   sudo bash celestia-init.sh --quick
#
set -euo pipefail

# ── Defaults ────────────────────────────────────────────────────────────────
NO_MIRROR=0
MIRROR=""
QUICK=0
ENTELECHEIA_WORKDIR="${HOME}/projects/celestia"
LOG_DIR="${HOME}/.local/share/celestia/logs"

DOCKER_IMAGES=(
    "pgvector/pgvector:pg18-bookworm"
)

# ── Helpers ─────────────────────────────────────────────────────────────────
c_info()  { printf '\033[1;34m[INIT]  %s\033[0m\n'  "$*" ; }
c_ok()    { printf '\033[1;32m[INIT]  %s\033[0m\n'  "$*" ; }
c_warn()  { printf '\033[1;33m[INIT]  %s\033[0m\n'  "$*" ; }
c_err()   { printf '\033[1;31m[INIT]  %s\033[0m\n' "$*" ; }
c_step()  { printf '\n\033[1;36m[INIT]  ==> %s\033[0m\n'   "$*" ; }

must_be_root() {
    if [[ "$(id -u)" -ne 0 ]]; then
        c_err "This script must run as root inside the WSL2 instance."
        c_err "Usage: sudo bash celestia-init.sh"
        exit 1
    fi
}

detect_china() {
    if curl -s --connect-timeout 3 --max-time 5 https://www.baidu.com >/dev/null 2>&1; then
        return 0
    fi
    return 1
}

# ── Argument parsing ────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --no-mirror) NO_MIRROR=1; shift ;;
        --mirror)    MIRROR="$2"; shift 2 ;;
        --quick)     QUICK=1; shift ;;
        -h|--help)
            echo "Usage: celestia-init.sh [--no-mirror] [--mirror URL] [--quick]"
            exit 0
            ;;
        *) c_err "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Step 1: Configure apt mirrors (China auto-detect) ───────────────────────
configure_apt_mirrors() {
    c_step "Step 1: Configuring apt sources"
    apt-get update -qq 2>/dev/null && { c_ok "apt update succeeded — no mirror needed."; return 0; }

    if ! detect_china; then
        c_warn "apt update failed but not in China. Check network."
        return 1
    fi

    c_info "Detected China network — switching to Aliyun apt mirror..."
    if [[ -f /etc/apt/sources.list.d/ubuntu.sources ]]; then
        # Ubuntu 24.04+ uses deb822 format
        sed -i 's|http://archive.ubuntu.com|http://mirrors.aliyun.com|g' /etc/apt/sources.list.d/ubuntu.sources 2>/dev/null || true
        sed -i 's|http://security.ubuntu.com|http://mirrors.aliyun.com|g' /etc/apt/sources.list.d/ubuntu.sources 2>/dev/null || true
    elif [[ -f /etc/apt/sources.list ]]; then
        sed -i 's|http://archive.ubuntu.com|http://mirrors.aliyun.com|g' /etc/apt/sources.list 2>/dev/null || true
        sed -i 's|http://security.ubuntu.com|http://mirrors.aliyun.com|g' /etc/apt/sources.list 2>/dev/null || true
    fi
    apt-get update -qq && c_ok "apt mirror configured (Aliyun)" || c_warn "apt mirror config may have failed"
}

# ── Step 2: Install Docker Engine ───────────────────────────────────────────
install_docker() {
    c_step "Step 2: Installing Docker Engine"

    if command -v docker >/dev/null 2>&1; then
        c_ok "Docker already installed: $(docker --version)"
        if docker info >/dev/null 2>&1; then
            c_ok "Docker daemon already running"
            return 0
        fi
        service docker start 2>/dev/null || systemctl start docker 2>/dev/null || true
        sleep 2
        if docker info >/dev/null 2>&1; then
            c_ok "Docker daemon started"
            return 0
        fi
    fi

    export DEBIAN_FRONTEND=noninteractive
    apt-get install -y -qq ca-certificates curl gnupg lsb-release

    # Try the linuxmirrors.cn one-click Docker installer first. This script
    # auto-detects the best mirror for both apt and Docker repos, solving
    # the "download.docker.com unreachable" problem in China.
    #
    # Source: https://linuxmirrors.cn/docker.sh — MIT-licensed, well-known
    # in the Chinese Linux community. We embed the URL (not the script body)
    # so users always fetch the latest mirror list.
    local docker_mirror_url="https://linuxmirrors.cn/docker.sh"
    local mirror_ok=false

    c_info "Attempting Docker install via linuxmirrors.cn (auto-mirror)..."
    if curl -fsSL --connect-timeout 10 --max-time 30 "$docker_mirror_url" -o /tmp/celestia-docker-install.sh 2>/dev/null; then
        if bash /tmp/celestia-docker-install.sh 2>&1; then
            c_ok "Docker installed via linuxmirrors.cn mirror"
            mirror_ok=true
        fi
        rm -f /tmp/celestia-docker-install.sh
    fi

    if $mirror_ok; then
        service docker start 2>/dev/null || systemctl enable --now docker 2>/dev/null || true
        if docker info >/dev/null 2>&1; then
            c_ok "Docker Engine installed and running"
            return 0
        fi
        c_warn "Docker installed but daemon may need a manual start."
        return 0
    fi

    # Fallback: manual Docker repo setup (works when download.docker.com is
    # reachable — e.g. outside China, or when a global proxy is configured).
    c_warn "linuxmirrors.cn unreachable — falling back to manual Docker install."
    c_warn "If this fails, set HTTP_PROXY/HTTPS_PROXY or pre-install Docker manually."

    install -m 0755 -d /etc/apt/keyrings
    curl -fsSL https://download.docker.com/linux/ubuntu/gpg | gpg --dearmor -o /etc/apt/keyrings/docker.gpg 2>/dev/null || true
    chmod a+r /etc/apt/keyrings/docker.gpg 2>/dev/null || true

    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] https://download.docker.com/linux/ubuntu $(lsb_release -cs) stable" > /etc/apt/sources.list.d/docker.list

    apt-get update -qq
    apt-get install -y -qq docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin fuse-overlayfs

    service docker start 2>/dev/null || systemctl enable --now docker 2>/dev/null || true

    if docker info >/dev/null 2>&1; then
        c_ok "Docker Engine installed and running"
    else
        c_warn "Docker Engine installed but daemon may need a manual start."
        c_warn "Try: sudo service docker start"
    fi
}

# ── Step 3: Configure Docker registry mirrors ───────────────────────────────
configure_docker_mirrors() {
    if [[ "$NO_MIRROR" -eq 1 ]]; then
        c_info "Step 3: Skipping Docker mirror (--no-mirror)"
        return 0
    fi
    c_step "Step 3: Configuring Docker registry mirrors"

    local china=false
    if detect_china; then china=true; fi

    local mirrors=()
    if [[ -n "$MIRROR" ]]; then
        mirrors+=("$MIRROR")
    elif $china; then
        mirrors+=(
            "https://docker.1ms.run"
            "https://docker.xuanyuan.me"
            "https://docker.m.daocloud.io"
        )
    else
        c_ok "Not in China — no mirror needed."
        return 0
    fi

    local mirror_json
    mirror_json=$(printf '%s\n' "${mirrors[@]}" | jq -R . | jq -s . 2>/dev/null || \
        python3 -c "import json,sys; print(json.dumps(sys.argv[1:]))" "${mirrors[@]}")

    mkdir -p /etc/docker
    local tmp
    tmp=$(mktemp)
    if [[ -f /etc/docker/daemon.json ]]; then
        if command -v jq >/dev/null 2>&1; then
            jq --argjson m "$mirror_json" '.["registry-mirrors"]=$m' /etc/docker/daemon.json > "$tmp"
        else
            python3 -c "import json; d=json.load(open('/etc/docker/daemon.json')); d['registry-mirrors']=$mirror_json; json.dump(d, open('$tmp','w'), indent=2)"
        fi
    else
        echo "{\"registry-mirrors\": $mirror_json}" > "$tmp"
    fi
    cp "$tmp" /etc/docker/daemon.json
    rm -f "$tmp"

    service docker restart 2>/dev/null || systemctl restart docker 2>/dev/null || true
    c_ok "Docker mirrors configured: ${mirrors[*]}"
}

# ── Step 4: Verify fuse-overlayfs ───────────────────────────────────────────
verify_fuse_overlayfs() {
    c_step "Step 4: Verifying fuse-overlayfs"
    if command -v fuse-overlayfs >/dev/null 2>&1; then
        c_ok "fuse-overlayfs: $(fuse-overlayfs --version 2>&1 | head -1)"
    else
        c_warn "fuse-overlayfs missing — installing..."
        apt-get install -y -qq fuse-overlayfs || c_err "Failed to install fuse-overlayfs"
    fi
}

# ── Step 5: Pull Docker images ──────────────────────────────────────────────
pull_images() {
    c_step "Step 5: Pulling Docker images"
    for img in "${DOCKER_IMAGES[@]}"; do
        c_info "Pulling: $img"
        if docker pull "$img" 2>/dev/null; then
            c_ok "Pulled: $img"
            continue
        fi
        local pulled=0
        for _ in 2 3; do
            sleep 2
            if docker pull "$img" 2>/dev/null; then
                c_ok "Pulled: $img (retry)"; pulled=1; break
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

# ── Step 6: Prepare workspace ───────────────────────────────────────────────
prepare_workspace() {
    c_step "Step 6: Preparing celestia workspace"

    mkdir -p "$ENTELECHEIA_WORKDIR"
    mkdir -p "$LOG_DIR"

    if [[ ! -f "${HOME}/.cargo/env" ]]; then
        c_info "Installing Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        c_ok "Rust installed"
    else
        c_ok "Rust already installed"
    fi

    c_ok "Workspace directories ready:"
    c_ok "  Projects: $ENTELECHEIA_WORKDIR"
    c_ok "  Logs:     $LOG_DIR"
}

# ── Step 7: Write instance discovery endpoint ───────────────────────────────
write_instance_toml() {
    c_step "Step 7: Writing instance discovery endpoint"

    local instance_id="${CELESTIA_INSTANCE_ID:-}"
    if [[ -z "$instance_id" ]]; then
        # Fallback: try to read from evernight config
        local ever_config="${HOME}/.config/evernight/config.toml"
        if [[ -f "$ever_config" ]]; then
            instance_id=$(grep -oP '^\s*id\s*=\s*\K\d+' "$ever_config" 2>/dev/null || true)
        fi
    fi
    if [[ -z "$instance_id" ]]; then
        # Last resort: generate a random ID
        instance_id=$((RANDOM % 1000))
        c_warn "No CELESTIA_INSTANCE_ID set — generated random id=${instance_id}"
    fi

    local instance_name
    instance_name=$(printf "celestia-%03d" "$instance_id")
    local scepter_port=$((8424 + instance_id * 100))

    local config_dir="${HOME}/.config/celestia"
    mkdir -p "$config_dir"

    cat > "${config_dir}/instance.toml" <<TOML
[instance]
id = ${instance_id}
name = "${instance_name}"

[scepter]
host = "localhost"
port = ${scepter_port}
health_url = "http://localhost:${scepter_port}/health"

[projects]
root = "${ENTELECHEIA_WORKDIR}"
mounted = ["entelecheia", "evernight", "shittim-chest", "arona", "noa", "scriptum"]
TOML

    c_ok "Wrote ${config_dir}/instance.toml"
    c_ok "  Instance: ${instance_name}  (id=${instance_id}, port=${scepter_port})"
}

# ── Summary ─────────────────────────────────────────────────────────────────
show_manifest() {
    local sep
    sep="$(printf '─%.0s' {1..60})"
    echo ""
    echo "$sep"
    echo "  celestia-init — complete"
    echo "$sep"
    echo ""
    echo "  Installed:"
    echo "    Docker:     $(docker --version 2>/dev/null || echo 'MISSING')"
    echo "    fuse-ovl:   $(fuse-overlayfs --version 2>&1 | head -1 || echo 'MISSING')"
    echo "    Rust:       $(rustc --version 2>/dev/null || echo 'MISSING')"
    echo ""
    echo "  Images in docker:"
    docker images --format '    {{.Repository}}:{{.Tag}}  {{.Size}}' 2>/dev/null || true
    echo ""
    echo "  Paths inside this instance:"
    echo "    Workspace:  $ENTELECHEIA_WORKDIR"
    echo "    Logs:       $LOG_DIR"
    echo ""
    echo "  This instance is ready for entelecheia builds."
    echo "$sep"
}

# ── Main ───────────────────────────────────────────────────────────────────
must_be_root

echo ""
printf '%.0s─' {1..60}; echo
echo "  celestia-init.sh — WSL2 instance initialization"
printf '%.0s─' {1..60}; echo
c_info "Hostname: $(hostname)"
c_info "Kernel:   $(uname -sr)"

configure_apt_mirrors || c_warn "apt mirror setup had issues; continuing anyway."
install_docker
configure_docker_mirrors
verify_fuse_overlayfs
pull_images
prepare_workspace
write_instance_toml
show_manifest
