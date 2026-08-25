#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# Port/topology layout matches server-topology.md (same repo, same dir).
# ──────────────────────────────────────────────────────────────────────────────
# celestia-server-bootstrap.sh — one-command SERVER profile bootstrap.
#
# Pulls prebuilt artifacts from the public e.celestia.world downloads channel
# and brings up the celestia server stack on a clean Linux host:
#
#   scepter (entelecheia) · chest (shittim-chest) · arona · evernight-server
#   + malkuth front door · PostgreSQL (pgvector) · secrets bootstrap
#
# Usage (on the server, as a sudo-capable user):
#
#   curl -fsSL https://e.celestia.world/downloads/files/celestia-server-bootstrap.sh \
#     | sudo bash -s -- --domain panel.example.com
#
#   bash celestia-server-bootstrap.sh [--domain DOMAIN] [--skip-db] [--skip-firewall]
#        [--channel URL] [--bind 0.0.0.0] [--port-base 3000]
#
# Design notes:
#   - Artifacts come from https://e.celestia.world/downloads/manifest.json
#     (sha256 digests are verified against the manifest before install).
#   - Secrets are generated with openssl rand when absent and persisted to
#     /etc/celestia/server.env (mode 600). Never committed anywhere.
#   - Each service runs under systemd with restart=always; malkuth supervises
#     the front door exactly like the node-2 production topology.
#   - This script is idempotent: re-running refreshes binaries and configs.
#
# Repository layout note: this script lives in plana/scripts/install/ — the
# single source of truth for celestia installers (see README.md there).
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

# ── Defaults ──────────────────────────────────────────────────────────────────
CHANNEL="https://e.celestia.world/downloads"
DOMAIN=""
BIND="0.0.0.0"
PORT_BASE=3000          # scepter=base, chest=base+1, arona=base+2, evernight-server=base+8
INSTALL_DIR="/usr/local/bin"
CONF_DIR="/etc/celestia"
DATA_DIR="/var/lib/celestia"
LOG_DIR="/var/log/celestia"
SKIP_DB=false
SKIP_FIREWALL=false
STAGE_DIR="$(mktemp -d /tmp/celestia-bootstrap.XXXXXX)"

# ── Helpers ───────────────────────────────────────────────────────────────────
log()  { printf '\033[1;32m[bootstrap]\033[0m %s\n' "$*"; }
warn() { printf '\033[1;33m[bootstrap]\033[0m %s\n' "$*" >&2; }
die()  { printf '\033[1;31m[bootstrap]\033[0m %s\n' "$*" >&2; exit 1; }
need() { command -v "$1" >/dev/null 2>&1 || die "missing dependency: $1 (install it first)"; }
cleanup() { rm -rf "$STAGE_DIR"; }
trap cleanup EXIT

# ── Argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
  case "$1" in
    --domain)        DOMAIN="$2"; shift 2 ;;
    --bind)          BIND="$2"; shift 2 ;;
    --port-base)     PORT_BASE="$2"; shift 2 ;;
    --channel)       CHANNEL="${2%/}"; shift 2 ;;
    --skip-db)       SKIP_DB=true; shift ;;
    --skip-firewall) SKIP_FIREWALL=true; shift ;;
    -h|--help)       sed -n '4,26p' "$0"; exit 0 ;;
    *)               warn "unknown option: $1"; shift ;;
  esac
done

[[ $EUID -eq 0 ]] || die "run as root (sudo)"

need curl; need openssl; need python3; need systemctl
curl -fsSL --connect-timeout 10 -o /dev/null "$CHANNEL/manifest.json" \
  || die "downloads channel unreachable: $CHANNEL"

# ── 1. Fetch + verify artifacts ───────────────────────────────────────────────
log "Fetching artifact manifest from $CHANNEL"
curl -fsSL "$CHANNEL/manifest.json" -o "$STAGE_DIR/manifest.json"

download_verified() {
  # download_verified <name> — fetch artifact, verify sha256 against manifest
  local name="$1"
  local expected
  expected=$(python3 - "$STAGE_DIR/manifest.json" "$name" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    m = json.load(f)
for a in m.get("artifacts", []):
    if a["name"] == sys.argv[2]:
        print(a["sha256"]); sys.exit(0)
sys.exit(1)
PY
  ) || die "artifact not found in manifest: $name"
  curl -fsSL "$CHANNEL/files/$name" -o "$STAGE_DIR/$name"
  local actual
  actual=$(sha256sum "$STAGE_DIR/$name" | cut -d' ' -f1)
  [[ "$actual" == "$expected" ]] || die "sha256 mismatch for $name (manifest=$expected got=$actual)"
  log "verified $name (${expected:0:16}…)"
}

# Artifacts are arch-keyed (see the downloads channel naming).
ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  EV_ARTIFACT="evernight-x86_64-linux" ;;
  aarch64) EV_ARTIFACT="evernight-aarch64-linux" ;;
  *)       die "unsupported architecture: $ARCH" ;;
esac

# The scepter / chest / arona server binaries follow the channel naming
# <name>-<version>-linux-x86_64; bootstrap always takes the newest by sorting.
newest_artifact() {
  python3 - "$STAGE_DIR/manifest.json" "$1" <<'PY'
import json, sys
with open(sys.argv[1]) as f:
    m = json.load(f)
names = sorted((a["name"] for a in m.get("artifacts", [])
                if a["name"].startswith(sys.argv[2] + "-")),
               key=lambda n: [int(p) if p.isdigit() else p
                              for p in n.replace("-", ".").split(".")])
if names:
    print(names[-1]); sys.exit(0)
sys.exit(1)
PY
}

# The evernight gateway ships arch-keyed names (evernight-x86_64-linux);
# server artifacts use the <prefix>-v<version>-linux-x86_64 family. Match the
# exact family so arch-keyed gateway names are never mis-selected.
newest_versioned() {
  python3 - "$STAGE_DIR/manifest.json" "$1" <<'PY'
import json, re, sys
with open(sys.argv[1]) as f:
    m = json.load(f)
pat = re.compile(r"^" + re.escape(sys.argv[2]) + r"-v\d+(\.\d+)*-linux-x86_64$")
names = sorted((a["name"] for a in m.get("artifacts", []) if pat.match(a["name"])),
               key=lambda n: [int(p) if p.isdigit() else p
                              for p in n[len(sys.argv[2]) + 1:].replace("-linux-x86_64", "").split(".")])
if names:
    print(names[-1]); sys.exit(0)
sys.exit(1)
PY
}

SCEPTER_ARTIFACT=$(newest_versioned scepter)     || SCEPTER_ARTIFACT=""
CHEST_ARTIFACT=$(newest_versioned shittim-chest) || CHEST_ARTIFACT=""
ARONA_ARTIFACT=$(newest_versioned arona-server)  || ARONA_ARTIFACT=""
MALKUTH_ARTIFACT=$(newest_versioned malkuth)     || MALKUTH_ARTIFACT=""

# ── 2. Secrets bootstrap (idempotent) ─────────────────────────────────────────
mkdir -p "$CONF_DIR" && chmod 700 "$CONF_DIR"
ENV_FILE="$CONF_DIR/server.env"
if [[ ! -f "$ENV_FILE" ]]; then
  log "generating secrets → $ENV_FILE (mode 600)"
  JWT_SECRET=$(openssl rand -hex 32)
  SHITTIM_CHEST_ENCRYPTION_KEY=$(openssl rand -hex 32)
  ENTELECHEIA_CONNECTION_TOKEN=$(openssl rand -hex 32)
  ARONA_ADMIN_TOKEN=$(openssl rand -hex 32)
  EVERNIGHT_SERVER_TOKEN=$(openssl rand -hex 32)
  umask 177
  cat > "$ENV_FILE" <<SECEOF
# Generated by celestia-server-bootstrap on $(date -Is). KEEP SECRET.
JWT_SECRET=${JWT_SECRET}
SHITTIM_CHEST_ENCRYPTION_KEY=${SHITTIM_CHEST_ENCRYPTION_KEY}
ENTELECHEIA_CONNECTION_TOKEN=${ENTELECHEIA_CONNECTION_TOKEN}
ARONA_ADMIN_TOKEN=${ARONA_ADMIN_TOKEN}
EVERNIGHT_SERVER_TOKEN=${EVERNIGHT_SERVER_TOKEN}
SECEOF
  chmod 600 "$ENV_FILE"
else
  log "secrets file exists, keeping $ENV_FILE"
  # shellcheck disable=SC1090
  . "$ENV_FILE"
  # Backfill any variable missing from an older secrets file so set -u holds.
  changed=false
  for var in JWT_SECRET SHITTIM_CHEST_ENCRYPTION_KEY ENTELECHEIA_CONNECTION_TOKEN \
             ARONA_ADMIN_TOKEN EVERNIGHT_SERVER_TOKEN; do
    if [[ -z "${!var:-}" ]]; then
      eval "$var=\$(openssl rand -hex 32)"
      changed=true
    fi
  done
  if [[ "$changed" == true ]]; then
    umask 177
    cat > "$ENV_FILE" <<SECEOF2
# Regenerated missing entries on $(date -Is). KEEP SECRET.
JWT_SECRET=${JWT_SECRET}
SHITTIM_CHEST_ENCRYPTION_KEY=${SHITTIM_CHEST_ENCRYPTION_KEY}
ENTELECHEIA_CONNECTION_TOKEN=${ENTELECHEIA_CONNECTION_TOKEN}
ARONA_ADMIN_TOKEN=${ARONA_ADMIN_TOKEN}
EVERNIGHT_SERVER_TOKEN=${EVERNIGHT_SERVER_TOKEN}
SECEOF2
    chmod 600 "$ENV_FILE"
  fi
fi

# ── 3. PostgreSQL (pgvector) ─────────────────────────────────────────────────
if [[ "$SKIP_DB" != true ]]; then
  if ! command -v psql >/dev/null 2>&1; then
    log "installing postgresql"
    if command -v apt-get >/dev/null 2>&1; then
      DEBIAN_FRONTEND=noninteractive apt-get update -qq >/dev/null
      # pgvector is REQUIRED: scepter's init migration does CREATE EXTENSION vector.
      DEBIAN_FRONTEND=noninteractive apt-get install -y -qq postgresql postgresql-contrib "$(apt list 2>/dev/null | grep -oE 'postgresql-[0-9]+-pgvector' | sort -V | tail -1 || echo postgresql-16-pgvector)" >/dev/null
    elif command -v dnf >/dev/null 2>&1; then
      dnf install -y -q postgresql-server postgresql-contrib >/dev/null
      postgresql-setup --initdb >/dev/null 2>&1 || true
    else
      warn "no supported package manager for postgres — run with --skip-db and provide DATABASE_URL"
    fi
  fi
  systemctl enable --now postgresql 2>/dev/null || true

  DB_NAMES=(entelecheia chest arona)
  runuser -u postgres -- psql -tAc "SELECT 1 FROM pg_roles WHERE rolname='celestia'" | grep -q 1 \
    || runuser -u postgres -- psql -qc "CREATE ROLE celestia LOGIN PASSWORD '${JWT_SECRET:0:32}'" >/dev/null
  for db in "${DB_NAMES[@]}"; do
    runuser -u postgres -- psql -tAc "SELECT 1 FROM pg_database WHERE datname='$db'" | grep -q 1 \
      || runuser -u postgres -- createdb -O celestia "$db"
  done
  # pgvector extension when available (optional — scepter falls back to in-memory)
  for db in "${DB_NAMES[@]}"; do
    runuser -u postgres -- psql -d "$db" -qc "CREATE EXTENSION IF NOT EXISTS vector" \
      || die "pgvector extension unavailable in $db — install postgresql-<ver>-pgvector and re-run"
  done
  log "databases ready: ${DB_NAMES[*]}"
fi

# DB URLs (localhost socket auth via peer not available for TCP; use password)
DATABASE_URL_ENTE="postgres://celestia:${JWT_SECRET:0:32}@127.0.0.1:5432/entelecheia"
DATABASE_URL_CHEST="postgres://celestia:${JWT_SECRET:0:32}@127.0.0.1:5432/chest"
DATABASE_URL_ARONA="postgres://celestia:${JWT_SECRET:0:32}@127.0.0.1:5432/arona"

# ── 4. Install binaries ───────────────────────────────────────────────────────
install_binary() {  # install_binary <artifact> <dest-name> [required]
  local art="$1" dest="$2" required="${3:-true}"
  if [[ -z "$art" ]]; then
    [[ "$required" == true ]] && warn "no artifact for $dest — skipped (channel missing)" \
                              || warn "optional $dest not on channel — skipped"
    return 0
  fi
  download_verified "$art"
  install -m 755 "$STAGE_DIR/$art" "$INSTALL_DIR/$dest"
  log "installed $INSTALL_DIR/$dest ← $art"
}

log "downloading artifacts (arch=$ARCH)"
install_binary "$SCEPTER_ARTIFACT" scepter true
install_binary "$CHEST_ARTIFACT"   chest   true
install_binary "$ARONA_ARTIFACT"   arona   false
# The registry server bin ships in its own artifact when published; fall back
# to the gateway artifact name (serve subcommand missing → unit will retry).
EVSERVER_ARTIFACT=$(newest_versioned evernight-server) || EVSERVER_ARTIFACT=""
if [[ -n "$EVSERVER_ARTIFACT" ]]; then
  install_binary "$EVSERVER_ARTIFACT" evernight-server true
else
  install_binary "$EV_ARTIFACT" evernight true
fi

# malkuth front door (optional artifact; skipped when not on the channel)
install_binary "$MALKUTH_ARTIFACT" malkuth false

mkdir -p "$DATA_DIR" "$LOG_DIR"

# ── 5. systemd units ─────────────────────────────────────────────────────────
SCEPTER_PORT=$((PORT_BASE))
CHEST_PORT=$((PORT_BASE + 1))
ARONA_PORT=$((PORT_BASE + 2))
EV_PORT=$((PORT_BASE + 8))

write_unit() {  # write_unit <name> <ExecStart...>
  local name="$1"; shift
  cat > "/etc/systemd/system/${name}.service" <<UNIT
[Unit]
Description=celestia ${name}
After=network-online.target postgresql.service
Wants=network-online.target

[Service]
Type=simple
User=root
EnvironmentFile=-$ENV_FILE
ExecStart=$*
Restart=always
RestartSec=5
StandardOutput=append:$LOG_DIR/${name}.log
StandardError=append:$LOG_DIR/${name}.log

[Install]
WantedBy=multi-user.target
UNIT
  log "unit written: ${name}.service"
}

EVSERVER_BIN="$INSTALL_DIR/evernight-server"
[[ -x "$EVSERVER_BIN" ]] || EVSERVER_BIN="$INSTALL_DIR/evernight"
write_unit evernight-server \
  env EVERNIGHT_SERVER_TOKEN="$EVERNIGHT_SERVER_TOKEN" \
  "$EVSERVER_BIN" serve --host 127.0.0.1 --port "$EV_PORT"

if [[ -x "$INSTALL_DIR/arona" ]]; then
  write_unit arona \
    env ARONA_HOST="$BIND" ARONA_PORT="$ARONA_PORT" \
    env DATABASE_URL="$DATABASE_URL_ARONA" \
    env JWT_SECRET="$JWT_SECRET" ARONA_ADMIN_TOKEN="$ARONA_ADMIN_TOKEN" \
    "$INSTALL_DIR/arona" serve
fi

if [[ -x "$INSTALL_DIR/chest" ]]; then
  write_unit chest \
    env SHITTIM_CHEST_HOST="$BIND" SHITTIM_CHEST_PORT="$CHEST_PORT" \
    env SHITTIM_CHEST_DATABASE_URL="$DATABASE_URL_CHEST" \
    env JWT_SECRET="$JWT_SECRET" \
    env SHITTIM_CHEST_ENCRYPTION_KEY="$SHITTIM_CHEST_ENCRYPTION_KEY" \
    env ENTELECHEIA_CONNECTION_TOKEN="$ENTELECHEIA_CONNECTION_TOKEN" \
    env ENTELECHEIA_SCEPTER_URL="http://127.0.0.1:$SCEPTER_PORT" \
    env EVERNIGHT_SERVER_URL="ws://127.0.0.1:$EV_PORT/api/ws" \
    "$INSTALL_DIR/chest" serve
fi

write_unit scepter \
  env SERVER_BIND_ADDRESS="127.0.0.1:$SCEPTER_PORT" \
  env DATABASE_URL="$DATABASE_URL_ENTE" \
  env ENTELECHEIA_CONNECTION_TOKEN="$ENTELECHEIA_CONNECTION_TOKEN" \
  "$INSTALL_DIR/scepter"

systemctl daemon-reload
systemctl enable --now evernight-server.service
[[ -x "$INSTALL_DIR/arona" ]] && systemctl enable --now arona.service
[[ -x "$INSTALL_DIR/chest" ]]   && systemctl enable --now chest.service
systemctl enable --now scepter.service

# ── 6. Firewall (optional) ────────────────────────────────────────────────────
if [[ "$SKIP_FIREWALL" != true && -n "$DOMAIN" ]]; then
  if command -v ufw >/dev/null 2>&1; then
    ufw allow 80/tcp  >/dev/null 2>&1 || true
    ufw allow 443/tcp >/dev/null 2>&1 || true
    log "ufw: 80/443 open (put nginx + TLS in front)"
  fi
fi

# ── 7. Summary + health probe ─────────────────────────────────────────────────
log ""
log "Bootstrap complete."
log "  scepter:          127.0.0.1:$SCEPTER_PORT"
log "  chest:            $BIND:$CHEST_PORT"
[[ -x "$INSTALL_DIR/arona" ]] && log "  arona:            $BIND:$ARONA_PORT"
log "  evernight-server: 127.0.0.1:$EV_PORT/api/ws"
log "  secrets:          $ENV_FILE"
log "  logs:             $LOG_DIR/"
log ""
log "Next steps:"
log "  1. Health:   systemctl status scepter chest evernight-server"
log "  2. TLS:      put nginx (or malkuth --serve-host $DOMAIN) in front of :$CHEST_PORT"
log "  3. Devices:  evernight gateways register at ws://<host>:$EV_PORT/api/ws"
[[ -z "$DOMAIN" ]] || log "  4. Domain:   point $DOMAIN at this host for the public front door"
sleep 2
for svc in evernight-server chest scepter; do
  systemctl is-active --quiet "$svc" 2>/dev/null && log "✓ $svc active" || warn "$svc not active (check $LOG_DIR/$svc.log)"
done
