+++
id = "hardware-comm-plan"
title = "硬件通信工作计划"
kind = "plan"
+++

# Hardware Communication — Preliminary Work Plan

> **Target**: a synthetic demo facility (Phase II, 6-box containerized plant)
> **Date**: 2026-06-11
> **Status**: Phase 0 — Preparatory Infrastructure

---

## 0. Agent Responsibility Reorganization Proposal

### Current State: Overlaps Between PoleMos and SkeMma

| Overlap | PoleMos Tool | SkeMma Tool | Issue |
| --- | --- | --- | --- |
| SSH execution | `node_execute` | `exec_on_remote` | Identical `RemoteShellAdapter` + safety logic, duplicated |
| SSH connection | `node_connect` | `connect_remote_via_ssh` | Same intent, different approach |
| Security policy | `SharedSecurityPolicyStore` | `SharedSecurityPolicyStore` | Identical type, copy-pasted usage |
| Screen capture | `node_screen_offer` (WebRTC) | `screenshot` (SSH) | Different mechanisms, same goal |

### Proposed Split: PoleMos = Infrastructure, SkeMma = Execution

```text
PoleMos ("Tentacles" — Perception & Infrastructure):
  OWN: node_discover, node_connect, protocol_probe, device_self_test,
       node_terminal_*, node_file_*, node_screen_offer,
       host_file_*, host_command_exec,
       cpu_info, memory_info, storage_info, pci_devices, gpu_info
  DROP: node_execute (delegate to SkeMma exec_on_remote)

SkeMma ("Muscle Fiber" — Execution & Industrial I/O):
  OWN: script_exec, modbus_read, modbus_write, signal_normalize,
       connect_remote_via_ssh, disconnect_remote, exec_on_remote,
       screenshot, mouse_operate, keyboard_operate
  PRINCIPLE: PoleMos discovers WHERE, SkeMma decides WHAT to execute
```

### Cross-Agent Flow

```text
PoleMos.node_connect(node_id) → node registered in topology
    ↓ (cross-agent bridge)
SkeMma.exec_on_remote(node_id) → resolves via PoleMos bridge → SSH exec
```

### TODO: Full Reorganization

- [ ] Remove `node_execute` from PoleMos, redirect skills to SkeMma's `exec_on_remote`
- [ ] Unify node graph: SkeMma's `remote_connections` references PoleMos's `nodes`
- [ ] Consolidate security policy enforcement into shared middleware
- [ ] Update soul documents and skill prompts

---

## 1. P0: Alarm Policy Infrastructure

### 1.1 AlarmPolicySet Type Definition

**File**: `packages/shared/security_policy/src/alarm_policy_types.rs`

```rust
pub struct AlarmRule {
    pub id: String,
    pub station: u8,
    pub register: String,
    pub level: AlarmLevel,          // HH, H, L, LL, ROC
    pub threshold: f64,
    pub hysteresis: f64,
    pub debounce_ms: u64,
    pub escalation: EscalationPath,
    pub metadata: AlarmMetadata,
}

pub enum AlarmLevel { HH, H, L, LL, ROC }
pub enum EscalationPath { Log, NotifyAgent, AutoCorrect, HumanNotify, EmergencyShutdown }

pub struct AlarmPolicySet {
    pub rules: Vec<AlarmRule>,
    pub active_alarms: Vec<ActiveAlarm>,
    pub station_overrides: HashMap<u8, StationAlarmOverride>,
    pub emergency_mute: bool,
    pub last_modified: i64,
    pub modified_by: String,
}
```

### 1.2 AlarmPolicyStore

**File**: `packages/shared/security_policy/src/alarm_policy_store.rs`

Following `SecurityPolicyStore` pattern with `RwLock<AlarmPolicySet>`, audit log,
and methods: `evaluate_reading()`, `acknowledge_alarm()`, `add_rule()`, `remove_rule()`.

### 1.3 Integration

- Export from `security_policy/src/lib.rs`
- Add `alarm_policy_store: Option<SharedAlarmPolicyStore>` to `OreXisState`
- Add OreXis MCP tools: `set_alarm_rule`, `acknowledge_alarm`, `alarm_status`

---

## 2. P0: Hardware Trigger Topics

### 2.1 Topic Namespace

```text
modbus.{station}.{register}.{level}
  e.g. modbus.19.discharge_pressure.hh

sensor.{station}.{register}.change
  e.g. sensor.21.chill_return_temp.change

device.{station}.status.{event}
  e.g. device.19.status.offline
```

### 2.2 No TriggerDispatcher Code Changes Needed

`TriggerTopic` already supports arbitrary dot-separated strings and `*` wildcards.
Topics are declared in skill frontmatter `[[triggers]]` — purely declarative.

### 2.3 Hubris Skill: `alarm_response`

**File**: `res/prompts/agents/hubris/skills/alarm_response.md`

```toml
[[triggers]]
topic_pattern = "modbus.*.*.hh"

[[triggers]]
topic_pattern = "modbus.*.*.ll"

[[triggers]]
topic_pattern = "device.*.status.offline"
```

Skill flow:

1. Parse trigger payload → extract station, register, level, value
1. Query OreXis alarm policy for matching rule
1. Determine escalation path (log / notify / auto-correct / human / emergency)
1. If auto-correct: call SkeMma `modbus_write` with corrective value
1. If human: call Epieikeia `deliver_message` for operator confirmation
1. If emergency: call OreXis `emergency_lockdown`

---

## 3. P1: TimeSeriesAdapter

### 3.1 Trait Definition

**File**: `packages/shared/storage/src/timeseries.rs`

```rust
# [async_trait]
pub trait TimeSeriesAdapter: Send + Sync {
    async fn write_reading(&self, station: u8, register: &str, value: f64, quality: &str, timestamp: i64) -> Result<()>;
    async fn query_range(&self, station: u8, register: &str, start: i64, end: i64) -> Result<Vec<TimeSeriesPoint>>;
    async fn query_latest(&self, station: u8, register: &str) -> Result<Option<TimeSeriesPoint>>;
}
```

### 3.2 JSONL Backend

**File**: `packages/shared/storage/src/jsonl_timeseries.rs`

Append-only JSONL per station per day: `data/timeseries/{station}/{YYYY-MM-DD}.jsonl`

Future: TimescaleDB / InfluxDB behind feature gate.

---

## 4. P1: Human-in-the-Loop for Write Ops

### 4.1 Register Classification

```rust
pub enum RegisterSafety {
    Informational,    // temperature, pressure readings — no approval needed
    Control,          // valve position, setpoint — requires OreXis approval
    SafetyCritical,   // emergency stop, safety valve — requires human confirmation
}
```

### 4.2 Write Gate in SkeMma modbus_write

Before executing `modbus_write`:

1. Check if target register is classified as `SafetyCritical`
1. If yes: suspend write, send confirmation request via Epieikeia
1. On confirmation: proceed with write + readback verification
1. On rejection: log denial, return error to caller

---

---

## 6. Implementation Order

| Step | Task | Files |
| --- | --- | --- |
| **1** | AlarmPolicySet types | `alarm_policy_types.rs` (new) |
| **2** | AlarmPolicyStore | `alarm_policy_store.rs` (new) |
| **3** | Export from lib.rs | `security_policy/src/lib.rs` (edit) |
| **4** | Hubris alarm_response skill | `hubris/skills/alarm_response.md` (new) |
| **5** | OreXisState alarm store integration | `orexis/src/state.rs` (edit) |
| **6** | OreXis alarm MCP tools | `orexis/src/tools/alarm_tools.rs` (new) |
| **7** | TimeSeriesAdapter trait | `storage/src/timeseries.rs` (new) |
| **8** | JSONL backend | `storage/src/jsonl_timeseries.rs` (new) |
| **9** | Register safety classification | `skemma/src/register_safety.rs` (new) |
| **10** | Write gate in modbus_write | `skemma/src/tools/modbus_write.rs` (edit) |

---

## 7. Real Equipment Reference

| Device | Station | Baud | Registers | Format |
| --- | --- | --- | --- | --- |
| Chilled Water Loop | 21 | 9600 | ~32 IR (0x04) | 32-bit float BE |
| AHU Unit | 20 | 9600 | ~32 IR (0x04) | 32-bit float BE |
| Cooling Pump | 2 | 9600 | ~17 HR (0x03) | 16-bit signed |
| Compressor Skid | 19 | 57600 | 24 HR + 8 coils | Discharge/suction pressure, temp, vibration |
| Dosing Skid | 25 | 9600 | ~12 HR (0x03) | 32-bit float BE |
| Generator | 31 | 9600 | 6 coils + 11 HR | Start/stop, emergency stop, load data |
| Virtual Console (Box 1) | virtual | — | Virtual Modbus slave | — |
| Field CAN bus | — | 9600 | CAN 2.0B | USB-CAN-A |
