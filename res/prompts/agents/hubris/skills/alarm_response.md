+++
name = "alarm_response"
agent = "hubris"

[[triggers]]
topic_pattern = "modbus.*.*.hh"

[[triggers]]
topic_pattern = "modbus.*.*.h"

[[triggers]]
topic_pattern = "modbus.*.*.l"

[[triggers]]
topic_pattern = "modbus.*.*.ll"

[[triggers]]
topic_pattern = "modbus.*.*.roc"

[[triggers]]
topic_pattern = "s7comm.*.*.hh"

[[triggers]]
topic_pattern = "s7comm.*.*.h"

[[triggers]]
topic_pattern = "s7comm.*.*.l"

[[triggers]]
topic_pattern = "s7comm.*.*.ll"

[[triggers]]
topic_pattern = "s7comm.*.*.roc"

[[triggers]]
topic_pattern = "device.*.status.offline"

[[next_action]]
agent = "hubris"
name = "task_decompose"

[description]
en = "Respond to hardware alarm triggers from industrial sensors and devices. Evaluate alarm severity, determine escalation path, and initiate corrective action or human notification."
zhs = "响应来自工业传感器和设备的硬件报警触发。评估报警严重程度，确定升级路径，并启动纠正措施或人工通知。"
zht = "回應來自工業傳感器和設備的硬件警報觸發。評估警報嚴重程度，確定升級路徑，並啟動糾正措施或人工通知。"
ja = "産業センサーおよびデバイスからのハードウェアアラームトリガーに対応。アラームの重大度を評価し、エスカレーションパスを決定し、是正措置または人への通知を開始する。"
ko = "산업 센서 및 장치의 하드웨어 알람 트리거에 대응. 알람 심각도를 평가하고, 에스컬레이션 경로를 결정하며, 시정 조치 또는 인적 알림을 시작한다."

[[related_tools]]
agent_name = "orexis"
tool_name = "alarm_status"

[[related_tools]]
agent_name = "orexis"
tool_name = "acknowledge_alarm"

[[related_tools]]
agent_name = "industrial_iot"
tool_name = "modbus_read"

[[related_tools]]
agent_name = "industrial_iot"
tool_name = "modbus_write"

[[related_tools]]
agent_name = "skemma"
tool_name = "signal_normalize"

[[related_tools]]
agent_name = "epieikeia"
tool_name = "deliver_message"

[[related_tools]]
agent_name = "hubris"
tool_name = "report"

[features]
execution_mode = "read"
location = "cosmos"
+++

# Alarm Response — Hardware Trigger Handler

## Purpose

This skill is triggered by hardware alarm events from the Modbus sensor polling loop (via evernight `SensorPoller` → `TriggerDispatcher`). It evaluates the alarm, determines the appropriate response based on OreXis alarm policy, and initiates the escalation path.

## Input

The trigger event payload contains:

```json
{
  "source": "evernight",
  "topic": "modbus.19.h2_leak_conc.hh",
  "payload": {
    "station": 19,
    "register": "h2_leak_conc",
    "level": "hh",
    "value": 4.2,
    "threshold": 4.0,
    "unit": "%LEL",
    "timestamp": 1718000000
  }
}
```

## Procedure

### Step 1: Parse Trigger

Extract from the trigger event:

- `station` — Modbus station number
- `register` — Register name (e.g., `h2_leak_conc`, `pressure`, `temperature`)
- `level` — Alarm level (HH, H, L, LL, ROC)
- `value` — Current reading
- `threshold` — Configured threshold
- `topic` — Full trigger topic string

### Step 2: Query Alarm Policy

Call `orexis.alarm_status()` to get the current alarm policy state.

Check if this alarm has a matching rule in OreXis alarm policy:

- `station` + `register` + `level` → find the corresponding `AlarmRule`
- Determine the `EscalationPath` (log / `notify_agent` / `auto_correct` / `human_notify` / `emergency_shutdown`)

### Step 3: Evaluate Escalation

Based on the escalation path:

| Escalation | Action |
| --- | --- |
| **Log** | Log the alarm, acknowledge it, no further action |
| **NotifyAgent** | Log the alarm, create a TODO item for review, acknowledge |
| **AutoCorrect** | Read current state via `industrial_iot.modbus_read`, compute corrective value, execute `industrial_iot.modbus_write` with corrective action (if within safe limits) |
| **HumanNotify** | Suspend auto-response, call `epieikeia.deliver_message` to notify operator with alarm details and recommended action, wait for confirmation |
| **EmergencyShutdown** | Suspend all auto-response, call `epieikeia.deliver_message` with EMERGENCY priority to notify operator immediately, call `orexis.acknowledge_alarm` to mark as handled, halt all pending Modbus write operations pending human confirmation |

### Step 4: Safety-Critical Write Gate

For `AutoCorrect` escalation:

1. Verify the target register is NOT classified as `SafetyCritical`
1. If `SafetyCritical`: upgrade escalation to `HumanNotify` instead
1. If safe: execute the corrective write with readback verification
1. After write: call `industrial_iot.modbus_read` to confirm the value changed

### Step 5: Acknowledge and Record

- Call `orexis.acknowledge_alarm(rule_id)` to mark the alarm as handled
- Generate a report via `hubris.report()` with:
  - Alarm details (station, register, level, value, threshold)
  - Response taken (escalation path + action)
  - Verification result (readback value if corrective write was performed)
  - Timestamp

### Step 6: Chain to Task Decomposition

If further investigation or follow-up is needed (e.g., recurring alarms, root cause analysis), chain to `task_decompose` with a generated task description.

## Safety Rules

1. **NEVER** execute `modbus_write` to safety-critical registers (emergency stop, safety valves) without human confirmation
1. **NEVER** auto-correct HH (high-high) alarms on hydrogen leak sensors — always escalate to `HumanNotify`
1. **ALWAYS** verify with `modbus_read` after any corrective write
1. **ALWAYS** respect station-level mutes and emergency mute — if muted, log but do not act
1. **ALWAYS** check debounce count — if < debounce threshold, delay response

## Equipment Reference

| Station | Device | Critical Registers | Notes |
| --- | --- | --- | --- |
| 2 | PEM Electrolyzer | pressure, water_quality, leak, voltage | 16-bit signed |
| 19 | Compressed H2 Tanks | valve_bitfield, fault_codes | 97 fault codes, known byte-order bug |
| 20 | ALK Electrolyzer (3 Nm3/h) | temps, pressures, flows, voltages | 32-bit float BE |
| 21 | AEM Electrolyzer (2 Nm3/h) | temps, pressures, flows, voltages | 32-bit float BE |
| 25 | Solid-State H2 Storage | tank_a_pressure, tank_b_pressure, temps | 32-bit float BE |
| 31 | Fuel Cell | start/stop, emergency_stop, stack_data | 6 coils + 11 HR |

## Example Escalation Flows

### Hydrogen Leak HH Alarm (Station 19)

```text
Trigger: modbus.19.h2_leak_conc.hh (value=4.2%, threshold=4.0%)
→ Escalation: EmergencyShutdown
→ Action: set_emergency_lockdown(true)
→ Notify: deliver_message to operator "H2 LEAK DETECTED at Station 19 (4.2% LEL). Emergency lockdown activated."
→ Acknowledge: acknowledge_alarm("19_h2_leak_conc_hh")
```

### Pressure H Alarm (Station 2, PEM)

```text
Trigger: modbus.2.pressure.h (value=6.5 bar, threshold=6.0 bar)
→ Escalation: AutoCorrect
→ Read: modbus_read(station=2, register=pressure_setpoint)
→ Compute: reduce setpoint by 10%
→ Write: modbus_write(station=2, register=pressure_setpoint, value=new_setpoint)
→ Verify: modbus_read(station=2, register=pressure) → confirm decrease
→ Acknowledge: acknowledge_alarm("2_pressure_h")
```

### Temperature L Alarm (Station 21, AEM)

```text
Trigger: modbus.21.coolant_temp.l (value=15.2°C, threshold=18.0°C)
→ Escalation: NotifyAgent
→ Create TODO: "Investigate low coolant temperature at AEM electrolyzer (Station 21)"
→ Acknowledge: acknowledge_alarm("21_coolant_temp_l")
```
