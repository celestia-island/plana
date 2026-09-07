# Celestia server topology — declarative reference

> Source of truth for the server-side service layout. The production fleet
> (node-2 / node-3) follows this exactly; `celestia-server-bootstrap.sh`
> produces the same shape on a fresh host. When adding a service, update the
> port table in `PLAN.md` §0.5 *and* this document together.
>
> Verified against live `ss -tlnp` + `systemctl list-units` on 2026-08-25.

## Conventions

- Every business service is a **native binary supervised by malkuth** (never a
  bare systemd `ExecStart`): malkuth gives the rolling-restart-on-binary-swap,
  the info-landing front door, and the L4 sticky proxy in one unit.
- **Port bands** (see PLAN.md §0.5):
  - `3000–3011` — business listeners (per service, low ports on the LAN)
  - `3090–3097` — malkuth **L4 proxy** front ends (sticky; clients connect here)
  - `84xx`     — malkuth **info-landing** ports (TLS-terminated browser entry)
  - `5432–5434` — PostgreSQL
- malkuth unit shape (single binary, three roles):
  ```ini
  ExecStart=/usr/local/bin/malkuth \
    --host 127.0.0.1 \
    --proxy 3092:3003-3003 \      # L4 sticky proxy → pod port range
    --pod-count 1 \
    --info-port 8409 \            # info landing / nonce handshake
    --info-landing \
    --serve http://127.0.0.1:3003 \   # what the front door serves
    --serve-host e.celestia.world \
    --watch /usr/local/bin/e-celestia-world \   # binary swap → rolling restart
    --debounce 5 --drain-secs 5 \
    -- /usr/local/bin/e-celestia-world
  ```
- **WS device lane** (evernight): the malkuth `--serve-only` front door does
  **not** proxy WebSocket; device traffic goes **direct to the pod port**
  (3008). Put `/api/ws` on a direct nginx location when exposing it publicly.

## node-2 (192.168.2.65) — panel family

| Service | Binary | Pod port | Proxy | Info | Public host |
|---|---|---|---|---|---|
| chest | `/usr/local/bin/chest` | 3000 | 3091 | 8407 | dev.celestia.world |
| dev-celestia | `/srv/celestia/dev-celestia/dev-celestia` | 3005 | 3095 | 8413 | dev.celestia.world |
| e.celestia.world | `/usr/local/bin/e-celestia-world` | 3003 | 3092 | 8409 | e.celestia.world |
| arcaea | `/usr/local/bin/arcaea` | 3004 | 3090 | 8408 | arcaea.celestia.world |
| erp-celestia | `/srv/celestia/erp-celestia/erp` | 3006 | — | 8414 | erp.celestia.world |
| demo-mock | `/srv/celestia/demo-mock/chest-mock` | 3009 | 3097 | 8415 | demo.dev.celestia.world |
| gateway rescue (RPC) | node-2 /srv/celestia/gateway-rescue/gateway | 3014 | — | — | gateway.celestia.world/rescue/ (daemon nginx lane) |
| demo enrollment gateway | `/srv/celestia/demo-mock/evernight-gateway-demo` | 3013 | — | — | (LAN direct; nginx `demo.dev.cw/evernight/` internal-only) |
| evernight-server | `/usr/local/bin/evernight-server` | 3008 | — | 8412 | api.evernight.celestia.world |
| evernight host-agent | `/usr/local/bin/evernight` (host-serve) | 3007 | — | — | — |
| facility_sim ×3 | `facility_sim` | 1502–1504 | — | — | — |
| Postgres ×3 | docker | 5432/5433/5434 | — | — | — |

Downloads channel: `DOWNLOADS_DIR=/srv/celestia/downloads` (e.celestia.world
unit drop-in `20-downloads.conf`); served at `/downloads/` with
`manifest.json` digests.

## node-3 (192.168.2.64) — model / orchestration / edge

| Service | Binary | Port | Notes |
|---|---|---|---|
| scepter (entelecheia) | `~/.local/share/celestia/scepter` | 3000 | malkuth info 8410 / proxy 8412 |
| arona | `/usr/local/bin/arona` | 3002 | malkuth info 8406 / proxy 8421 |
| evernight bridge | `~/.local/share/celestia/evernight` (api-serve) | 3001 | — |
| evernight-sensor | `evernight sensor-poll --simulate` | — | pushes to scepter unix socket |
| evernight host-agent | `evernight host-serve` | — | registers on node-2 3008 |
| cep-llamacpp | `cep-llamacpp` | 3003 | — |
| cep-speech | python (sherpa-onnx) | 3004 | — |
| ollama | ollama | 11434 | `OLLAMA_MODELS=/mnt/work/ollama-models` |
| Postgres | local | 5432 | scepter + arona DBs |

## nginx (node-2) — TLS termination → malkuth front doors

| Host | proxy_pass |
|---|---|
| dev.celestia.world | 8413 (dev-celestia) |
| e.celestia.world | 8409 (e) |
| arcaea.celestia.world | 8408 (arcaea) |
| erp.celestia.world | 8414 (erp) |
| demo.dev.celestia.world | 8415 (demo) |
| api.evernight.celestia.world | `/api/ws` → 3008 direct; `/` → 8412 |

Public TLS terminates at the frp endpoint (LE cert `CN=e.celestia.world`); the
tunnel forwards cleartext to node-2 nginx:80. Internal nginx certs only cover
`dev.celestia.world` — test public paths with `--resolve <host>:443:<public-ip>`
and `--noproxy '*'`.


### 2026-09-07 delta (gateway RPC wave, zcode)

- `evernight-gateway-demo.service` on node-2: plain systemd unit (not yet
  malkuth-supervised), pod `0.0.0.0:3013`, env
  `/etc/celestia/evernight-gateway-demo.env` — `GATEWAY_CHEST_JWT_SECRET`
  aligned with the demo chest (`chest-demo-mock.env` JWT_SECRET), so demo
  flasher logins verify on it. Binary from evernight master #150 (the
  strict WS JSON-RPC `/api/rpc` surface). LAN clients reach it directly.
- **Public routing correction (same day, user-spotted):** the real public
  path for `demo.dev.celestia.world` is SakuraFrp → **the daemon nginx on
  :3000**, which Host-splits vhosts (`/etc/nginx/sites-enabled/chest` on
  the daemon: demo.dev.cw → node-2:8415, gateway.cw → 3011/3012, …) — the
  earlier note about the frp owner changing a mapping was a misdiagnosis.
  A `/evernight/` lane on that daemon vhost (→ node-2:3013, prefix
  stripped, WS-capable, backup `chest.bak.evernight-lane`) opens the
  public entry; verified end-to-end from the public HTTPS origin
  (gateway.info 200, WS upgrade 101 — WS needs HTTP/1.1; the earlier 400
  came from curl negotiating h2 by default, not a config fault). The
  node-2 `demo-dev` site keeps the same lane for LAN/origin-direct
  traffic.
- Observed drift vs the table above (2026-09-07 live check): pods
  3000–3012 are all in use; the production enrollment gateway
  (`evernight-gateway-malkuth.service`) actually serves pod **3011**
  (info 8417), not 3007 — 3007 belongs to the evernight host-agent line.

## PostgreSQL

| DB | Owner | Used by |
|---|---|---|
| entelecheia | celestia | scepter (pgvector) |
| chest | celestia | shittim-chest |
| arona | celestia | arona |

pgvector is a **hard** requirement (scepter's init migration runs
`CREATE EXTENSION vector`).

## Cross-node shared secrets

Single values shared by both nodes' drop-ins (never committed):

- `ENTELECHEIA_CONNECTION_TOKEN` — scepter ↔ chest trust
- `JWT_SECRET` — cloud-issuer tokens (e.celestia.world / chest / arona)
- `ARONA_ADMIN_TOKEN` — arona admin plane
- `EVERNIGHT_SERVER_TOKEN` — gateway registry bearer (fail-closed)

## Bootstrap equivalence

`celestia-server-bootstrap.sh --port-base 3000` produces, on one host:

| Role | Port |
|---|---|
| scepter | 3000 |
| chest | 3001 |
| arona | 3002 |
| evernight-server | 3008 |

with malkuth + nginx + TLS documented as the front-door step — the same
single-host projection of the two-node layout above.
