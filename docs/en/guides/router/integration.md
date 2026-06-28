# Integration Guide — Evernight

How to connect evernight to each supported protocol, what server software to
use, and how to verify the connection works end-to-end.

## Architecture

```
  Your app (CLI / TUI / Web / Agent)
         │
         ▼
   evernight crate
   ├── Industrial protocols (Modbus / S7comm / MC / EtherNet/IP / OPC UA / CAN / IPMI / EtherCAT)
   ├── Remote control (SSH / VNC / RDP)
   ├── Cloud (Proxmox / EC2 / k8s / libvirt / Tailscale / CODESYS / OpenPLC)
   ├── Security (vault / broker gates / write-approval)
   └── Tooling (MCP server / FFI / scripting / CLI)
         │
         ▼
   Physical hardware / remote servers / cloud APIs
```

---

## 1. Modbus RTU (Serial)

### Server side
- **Device**: any Modbus RTU slave (PLC, sensor, inverter)
- **No-server test**: `socat PTY,raw,echo=0,link=/tmp/vcom_a PTY,raw,echo=0,link=/tmp/vcom_b &`
  — creates a virtual serial pair; run `tests/modbus_slave_sim.rs` for 6 stations.

### Evernight side
```rust
use evernight::serial::modbus::ModbusMaster;

let master = ModbusMaster::builder(19)      // station 19
    .with_port("/dev/ttyUSB0")
    .with_baud_rate(57600)
    .with_timeout(2000)
    .open()?;

let result = master.read_registers(RegisterMode::Holding, 0x10, 33)?;
println!("Pressures: {:?}", &result.values[..3]);
```

### CLI
```bash
evernight sensor-poll --manifest corridor.toml
```

---

## 2. S7comm (Siemens)

### Server side
- **Device**: S7-1200/1500/300/400 PLC
- **Prerequisite**: TIA Portal → enable "Permit PUT/GET" + disable "Optimized block access"
- **No-hardware test**: `cargo test --features full --test s7comm_integration` (uses in-process snap7-server)

### Evernight side
```rust
use evernight::protocol::s7comm::{S7CommClient, S7ConnectParams};

let client = S7CommClient::new(S7ConnectParams {
    host: "192.168.1.10".into(), port: 102, rack: 0, slot: 1,
});
client.connect().await?;
let bytes = client.read_db(1, 0, 4).await?;       // DB1 offset 0, 4 bytes
let temp = f32::from_be_bytes(bytes.try_into().unwrap());
println!("Temperature: {:.1} °C", temp);
```

---

## 3. MC Protocol (Mitsubishi)

### Server side
- **Device**: MELSEC FX/Q/L/iQ-R PLC
- **No-hardware test**: `tests/mc_test_server.rs` (in-process mock MC server)

### Evernight side
```rust
use evernight::protocol::mc_protocol::{McProtocolClient, McDevice};

let client = McProtocolClient::new("192.168.1.5", 5000);
client.connect().await?;
let words = client.read_devices(McDevice::D, 0, 10).await?;
println!("D0-D9: {:?}", words);
```

---

## 4. EtherNet/IP (Rockwell)

### Server side
- **Device**: Allen-Bradley CompactLogix / ControlLogix
- **No-hardware test**: unit tests with hand-built frames (no live simulator)

### Evernight side
```rust
use evernight::protocol::ethernet_ip_backend::EthernetIpBackend;
use evernight::protocol::backend::{ProtocolBackend, TransportInfo, DataAddress};

let mut backend = EthernetIpBackend::new("192.168.1.10", 44818);
backend.connect(&TransportInfo::Tcp { host: "192.168.1.10".into(), port: 44818 })?;
let result = backend.read(&DataAddress::Raw {
    data: b"0x6E:0x01:0x05".to_vec(),  // class 0x6E, instance 1, attr 5
    size: 4,
})?;
println!("Value: {:02X?}", result.raw);
```

---

## 5. OPC UA

### Server side
- **Device/Software**: any OPC UA server (KEPServerEX, Ignition, CODESYS, etc.)
- **Self-hosted**: evernight itself can act as an OPC UA server:

```rust
use evernight::protocol::opcua_server::OpcUaSensorServer;

let mut server = OpcUaSensorServer::new("opc.tcp://0.0.0.0:4840", 4840)?;
let node = server.add_sensor_variable("temperature", 25.5)?;
// server.run();  // blocks — run in a separate thread
```

### Evernight client side
```rust
use evernight::protocol::opcua_client::{OpcUaClient, OpcUaEndpoint, OpcUaSecurity};

let endpoint = OpcUaEndpoint::new_anonymous("opc.tcp://192.168.1.50:4840");
let client = OpcUaClient::connect(&endpoint).await?;
let value = client.read_node("ns=2;s=Temperature").await?;
println!("Temperature: {}", value);
```

---

## 6. SSH

### Server side
- Any SSH server (OpenSSH, Dropbear, etc.)
- **Key management**: `evernight vault init ~/.config/evernight/vault "passphrase"`
  then add credentials.

### Evernight side
```bash
# Interactive terminal
evernight connect ssh://user@192.168.1.100

# One-shot command
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"

# File transfer
evernight file put ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# SOCKS5 proxy through SSH
evernight proxy 1080 --host 192.168.1.100 --user root
```

---

## 7. VNC

### Server side
- **Software**: TigerVNC, x11vnc, RealVNC, etc.
- **Install**: `apt install tigervnc-standalone-server && vncserver :1`
- **Browser access**: evernight can proxy VNC to WebSocket:

```bash
# Start a VNC-to-WebSocket proxy (noVNC-compatible)
# (programmatically via evernight::vnc::ws::serve_vnc_websocket)
```

### Evernight side
```bash
# CLI one-shot — handshake + server info + one frame capture
evernight connect vnc://192.168.1.100:5901
```

---

## 8. RDP

### Server side
- **Windows**: native RDP (Settings → Remote Desktop → Enable)
- **Linux**: `apt install xrdp`
  - For TLS: set `security_layer=tls` in `/etc/xrdp/xrdp.ini`
  - Generate cert: `openssl req -x509 -newkey rsa:2048 -nodes -keyout key.pem -out cert.pem -days 365 -subj "/CN=localhost"`
  - Copy to `/etc/xrdp/cert.pem` + `/etc/xrdp/key.pem`

### Evernight side
```rust
use evernight::rdp::x224::{RdpClient, RdpConfig};

let config = RdpConfig {
    host: "192.168.1.100".to_string(),
    port: 3389,
    username: Some("admin".into()),
    password: Some("password".into()),
    ..Default::default()
};
let mut client = RdpClient::connect(&config).await?;
println!("Connected: {}x{}, TLS", client.width(), client.height());
// Send MCS Connect-Initial, Attach-User, Channel-Join, ...
// Receive bitmap updates, decode to RGBA via bitmap::decode_to_rgba
```

```bash
# CLI one-shot — handshake + protocol/desktop info
evernight connect rdp://192.168.1.100:3389
```

### Where RDP is today
- ✅ Transport: X.224 + TLS upgrade (verified against real xrdp)
- ✅ MCS: Connect-Initial/Response + Attach-User (verified)
- ✅ Bitmap: uncompressed + Interleaved RLE decode → RGBA
- ✅ Input: keyboard scancodes + mouse events
- ✅ Channels: CLIPRDR + RDPDR + RDPSND + DVC
- ✅ NLA: NTLMv2 + CredSSP (Kerberos needs KDC)
- ◐ Session: needs Channel-Join → capability exchange → continuous framebuffer loop

---

## 9. Kubernetes

### Server side
- Any k8s cluster (minikube, kind, EKS, GKE, etc.)
- Auth: `~/.kube/config` or in-cluster service account

### Evernight side
```rust
use evernight::cloud::k8s::K8sClient;

let client = K8sClient::from_kubeconfig("default").await?;
let pods = client.list_pods().await?;
for pod in &pods {
    println!("{}: {} ({} containers)", pod.name, pod.phase, pod.containers.len());
}
```

---

## 10. libvirt

### Server side
- `apt install libvirt-daemon-system libvirt-dev`
- Start libvirtd: `systemctl start libvirtd`

### Evernight side
```rust
use evernight::cloud::libvirt_client::LibvirtClient;

let client = LibvirtClient::open_read_only("qemu:///system")?;
let domains = client.list_domains()?;
for d in &domains {
    println!("{}: {:?}", d.name, d.state);
}
```

---

## The Client Question: Should You Build a Viewer?

### The problem

Evernight decodes RDP bitmaps to RGBA buffers and VNC frames to pixels, but
**nowhere to display them**. Without a renderer:

- You can't visually verify that bitmap decoding is correct (you can only check
  it doesn't crash)
- You can't dogfood — evernight is supposed to be an "XPipe-class universal
  connection manager", and XPipe shows remote desktops
- Manual testing requires an external viewer (mstsc.exe / xfreerdp / vinagre)

### Recommendation: YES — build a minimal embedded viewer

Three tiers of effort, from simplest to most useful:

#### Tier 1: Headless screenshot (smallest effort, highest test value)

```
evernight connect rdp://host --screenshot out.png
```

Captures ONE frame after session setup, writes it to a PNG. No GUI needed.
Uses the existing `bitmap::decode_to_rgba` + a simple PNG encoder (or PPM,
which needs zero deps). This gives you:

- **Automated visual regression**: compare screenshots across commits
- **Protocol correctness**: you can SEE if the RDP bitmap decode is right
- **CI-friendly**: no display server required

Estimated effort: ~100 lines (PNG encode + one-shot capture loop).

#### Tier 2: egui window (moderate effort, full manual testing)

```
evernight connect rdp://host --gui
```

Opens an [egui](https://github.com/emilk/egui) window showing the live RDP
framebuffer. Keyboard/mouse input is sent back via the existing input codec.
This gives you:

- **Full closed-loop**: type → see output → verify interaction
- **No external deps**: egui is pure Rust, cross-platform
- **Single binary**: no separate viewer app needed

Estimated effort: ~300 lines (egui texture upload + input event loop).
The egui `eframe` crate is already common in the ecosystem.

#### Tier 3: Web frontend via existing API (largest effort, production)

Evernight already has `api-serve --transport ws` (JSON-RPC over WebSocket).
A web frontend (shittim-chest / Tauri) connects to this API and:

- Renders the framebuffer on a `<canvas>`
- Sends input events via JSON-RPC
- This is the production path — Tier 1+2 are for development/testing

This is frontend work (Vue/React/Tauri), not evernight library code.

### Which tier to build?

**Start with Tier 1 (headless screenshot)** — it's the highest ROI for testing
and takes ~1 hour. It closes the most critical gap: you can finally SEE whether
the RDP bitmap pipeline produces correct pixels.

Then add Tier 2 (egui) when you need interactive testing — e.g. verifying
keyboard input, clipboard, drive redirect.

Tier 3 is the production frontend, built when the web UI is ready.

---

## Quick-Start: Verify Your Setup

```bash
# 1. Build
cargo build --features full --release

# 2. Test all protocols (878 tests)
cargo test --features full

# 3. Probe a host for industrial protocols
evernight probe 192.168.1.20 --ports 502,102,4840,5000

# 4. Connect to an SSH host
evernight connect ssh://user@192.168.1.100

# 5. Poll sensors from a manifest
evernight sensor-poll --manifest corridor.toml

# 6. Check hardware telemetry
evernight hw

# 7. Start the MCP server (for AI agents)
evernight api-serve --transport ws --port 50000
```
