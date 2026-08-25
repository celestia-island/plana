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
