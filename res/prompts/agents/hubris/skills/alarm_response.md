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

[[related_tools]]
agent_name = "orexis"
tool_name = "alarm_status"

[[related_tools]]
agent_name = "orexis"
tool_name = "acknowledge_alarm"

[[related_tools]]
agent_name = "orexis"
tool_name = "verify_write_safety"

[[related_tools]]
agent_name = "orexis"
tool_name = "request_write_approval"

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
  "topic": "modbus.19.discharge_pressure.hh",
  "payload": {
    "station": 19,
    "register": "discharge_pressure",
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
- `register` — Register name (e.g., `discharge_pressure`, `pressure`, `temperature`)
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

### Step 4: Operator Approval Gate (MANDATORY for every corrective write)

For `AutoCorrect` and `HumanNotify` escalations that involve a write, run the approval gate BEFORE any `industrial_iot.modbus_write`:

1. Call `orexis.verify_write_safety({ station_id, protocol, address, value })` with the target register and proposed value
2. If the result is `Allowed` → proceed to execute the corrective write with readback verification
3. If the result is `Denied` (or the register is classified `SafetyCritical`) → call `orexis.request_write_approval({ station_id, protocol, address, field_name, current_value, proposed_value, unit, reason, timeout_secs })` — this tool BLOCKS until the operator responds in the web UI (default 120s timeout)
4. Inspect the approval result:
   - `approved: true` → execute `industrial_iot.modbus_write`, then `industrial_iot.modbus_read` to verify the value changed, then `orexis.acknowledge_alarm`
   - `approved: false` or `timed_out: true` → do NOT write. Log the denial, call `epieikeia.deliver_message` to inform the operator, and do NOT retry the same write in this chain

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

1. **NEVER** execute `modbus_write` to safety-critical registers (emergency stop, safety valves) without an `approved: true` response from `orexis.request_write_approval`
1. **NEVER** auto-correct HH (high-high) alarms on safety-critical gas sensors — always escalate to `HumanNotify` with the approval gate
1. **NEVER** retry a denied write in the same chain — a denial or timeout ends the corrective attempt; inform the operator instead
1. **ALWAYS** verify with `modbus_read` after any corrective write
1. **ALWAYS** respect station-level mutes and emergency mute — if muted, log but do not act
1. **ALWAYS** check debounce count — if < debounce threshold, delay response

## Equipment Reference

| Station | Device | Critical Registers | Notes |
| --- | --- | --- | --- |
| 2 | Refrigerant Dryer | pressure, dew_point, flow, status | 16-bit signed |
| 19 | Compressor Skid | discharge_pressure, bearing_temp, vibration | 24 HR |
| 20 | AHU Unit | supply_temp, return_temp, humidity | 32-bit float BE |
| 21 | Chilled Water Loop | chill_return_temp, flow_rate, valve_position | 32-bit float BE |
| 25 | Dosing Skid | tank_level, dose_rate, pump_status | 32-bit float BE |
| 31 | Generator | start/stop, emergency_stop, load_percent | 6 coils + 11 HR |

## Example Escalation Flows

### Gas Leak HH Alarm (Station 19)

```text
Trigger: modbus.19.discharge_pressure.hh (value=13.5 bar, threshold=13.0 bar)
→ Escalation: EmergencyShutdown
→ Action: set_emergency_lockdown(true)
→ Notify: deliver_message to operator "GAS LEAK DETECTED at Station 19 (13.5 bar). Emergency lockdown activated."
→ Acknowledge: acknowledge_alarm("19_discharge_pressure_hh")
```

### Pressure H Alarm (Station 2)

```text
Trigger: modbus.2.pressure.h (value=6.5 bar, threshold=6.0 bar)
→ Escalation: AutoCorrect
→ Read: modbus_read(station=2, register=pressure_setpoint)
→ Compute: reduce setpoint by 10%
→ Write: modbus_write(station=2, register=pressure_setpoint, value=new_setpoint)
→ Verify: modbus_read(station=2, register=pressure) → confirm decrease
→ Acknowledge: acknowledge_alarm("2_pressure_h")
```

### Temperature L Alarm (Station 21)

```text
Trigger: modbus.21.coolant_temp.l (value=15.2°C, threshold=18.0°C)
→ Escalation: NotifyAgent
→ Create TODO: "Investigate low coolant temperature at chilled water loop (Station 21)"
→ Acknowledge: acknowledge_alarm("21_coolant_temp_l")
```
