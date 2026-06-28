+++
title = "TIA Portal Prerequisites — Connecting evernight"
description = """How to prepare an S7-1200/1500 PLC once in TIA Portal so evernight can connect, self-network, and read/write the device with no further human intervention."""
lang = "en"
category = "guides"
subcategory = "router"
+++

# TIA Portal Prerequisites — Connecting evernight

> **Goal**: prepare your Siemens S7-1200/1500 PLC **once** in TIA Portal so
> that evernight can connect, self-network, and read/write the device with **no
> further human intervention**. This is a one-time configuration of CPU
> properties — your ladder/SCL program logic is never touched.

evernight speaks **two channels** to a Siemens PLC. Pick one based on what your
PLC exposes:

| Channel | Port | Access style | Needs TIA prep | Best for |
|---------|------|--------------|----------------|----------|
| **S7comm** | 102 | raw byte R/W of M / I / Q / DB | PUT/GET + non-optimized DBs | legacy, lean, no OPC UA license |
| **OPC UA** | 4840 | symbolic, self-describing | enable built-in server | **recommended** — auto-discovery, optimized DBs OK |

If you can enable OPC UA, prefer it: evernight **browses** the full symbolic
address space automatically and there is zero manual symbol entry.

---

## Path A — S7comm (raw register access)

### A.1 Enable PUT/GET communication

S7-1200/1500 block external S7 read/write by default.

1. Open the project in **TIA Portal**.
2. In the device/network view, **click the CPU**.
3. Properties → **Protection & Security → Connection mechanisms**.
4. Check **"Permit access with PUT/GET communication from remote partner"**.
5. Download the hardware configuration to the CPU.

### A.2 Make target DBs non-optimized

Optimized block access (the S7-1200/1500 default) has no fixed byte offset, so
absolute-address reads fail. For each DB evernight must read/write:

1. Right-click the DB → **Properties**.
2. Uncheck **"Optimized block access"**.
3. Recompile and download.

> M markers and the I/Q process images are always byte-addressable — no change
> needed there. This step only concerns DBs.

### A.3 Connect from evernight

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500: `rack=0, slot=1`
- S7-300: `slot=2`

Self-network in code:

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures now describe every readable DB
```

---

## Path B — OPC UA (recommended)

### B.1 Firmware & license prerequisites

- CPU firmware **V2.0+** (V2.5+ for OPC UA methods).
- A **SIMATIC OPC UA runtime license** sized to the CPU (assigned under CPU
  Properties → **Runtime licenses → OPC UA**). Required for compliance.

### B.2 Enable the OPC UA server

1. **Click the CPU** in the network/device view.
2. Properties → **OPC UA → General**: enter a server name.
3. Properties → **OPC UA → Server**: check **"Activate OPC UA server"**.
4. Assign the server to the **PROFINET interface** the client will reach.

### B.3 Expose the symbolic variables

- **OPC UA → Server → Server interface**: select **"Standard SIMATIC server
  interface"** so every symbolic tag/DB (including optimized DBs) is published
  automatically. (Custom interfaces let you hand-pick tags.)

### B.4 Authentication & security

- **OPC UA → Server → User authentication**: anonymous (for a trusted LAN) or
  username/password.
- **OPC UA → Server → Security**: pick a policy. `None` is easiest for a first
  connection; `Sign & Encrypt` for production.
- On firmware **V3.1+ with TIA V19+**, grant the **"OPC UA server access"**
  functional role / runtime right to the connecting user.

### B.5 Trust the client certificate

OPC UA clients present an X.509 certificate on connect; the PLC quarantines
unknown certs. After evernight's first connection attempt, accept it:

1. TIA Portal → **CPU → Certificates** (online), **or**
2. PLC **Web server** → "Communication with OPC UA clients", **or**
3. The CPU display's certificate manager.

Then **accept/trust** the evernight client certificate.

### B.6 (Optional) Export the OPC UA NodeSet XML

A NodeSet file is an offline map of every variable — useful to pre-plan
without a live connection.

1. CPU Properties → **OPC UA → Server → Export**.
2. Click **"Export OPC UA XML file"**, save the `*.Opc.Ua.NodeSet2.xml`.

### B.7 Download

Download the **hardware configuration**. These are CPU properties, not program
logic — your ladder/SCL code is untouched.

### B.8 Connect from evernight

Endpoint URL:

```
opc.tcp://192.168.1.10:4840
```

evernight connects as an OPC UA client, **browses** the whole symbolic tree,
and reads/writes by name — no manual symbol entry, optimized DBs included.

---

## Verifying connectivity (zero-risk probes)

Before driving outputs, confirm the channels are alive with read-only probes:

```bash
# Is anything speaking S7comm on port 102?
evernight probe 192.168.1.10 --ports 102

# Is the OPC UA server up on 4840?
evernight probe 192.168.1.10 --ports 4840
```

Both are passive handshakes — they read nothing and write nothing.

---

## Safety boundaries

- **Never** put safety interlocks (e-stops, limits, overload) on the S7/OPC UA
  path. Keep them in the PLC scan. A dropped network link must not disable a
  safety function.
- evernight control is suited to **slow-acting** loads (valves, state machines,
  mode switches). Round-trip latency over S7/OPC UA is ~10–50 ms — fine for
  supervision, too slow for motion/servo.
- Prefer writing **command M-bits** that existing PLC logic acts on (you hijack
  the trigger source) over writing Q outputs directly.

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|--------------|-----|
| S7 connect refused / no COTP confirm | PUT/GET not enabled; wrong rack/slot; firewall | A.1; verify `rack=0 slot=1` (1200/1500) |
| DB read returns "optimized" / InvalidAddress | Optimized block access on | A.2 — uncheck optimized access, recompile |
| OPC UA endpoint unreachable | Server not activated; not downloaded; license missing | B.2 / B.7 / B.1 |
| OPC UA connects then rejected | Client certificate not trusted | B.5 |
| Browse returns empty | Standard SIMATIC interface not enabled | B.3 |

---

## References

- [Enabling the OPC UA server (S7-1500) — STEP 7 V20 docs](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [Access to the OPC UA server (endpoint URL / port 4840)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [Export OPC UA XML file (NodeSet)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
