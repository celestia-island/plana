+++
name = "device_interaction_sop"
agent = "skemma"

[[next_action]]
agent = "skopeo"
name = "node_task_summary"

[description]
en = "Standard Operating Procedure template for safe physical device interaction following the self-discovery protocol"
zh-Hans = "物理设备安全交互标准操作流程模板，遵循自发现协议的三阶段规范"
zh-Hant = "物理設備安全互動標準操作流程模板，遵循自發現協議的三階段規範"
ja = "自己発見プロトコルに従う、安全な物理デバイス操作のための標準操作手順テンプレート"
ko = "자가 발견 프로토콜을 따르는 안전한 물리적 장치 상호 작용을 위한 표준 운영 절차 템플릿"
fr = "Modèle de procédure opérationnelle standard pour l'interaction sécurisée avec les appareils physiques suivant le protocole d'auto-découverte"
es = "Plantilla de procedimiento operativo estándar para la interacción segura con dispositivos físicos siguiendo el protocolo de auto-descubrimiento"
ru = "Шаблон стандартной операционной процедуры для безопасного взаимодействия с физическими устройствами в соответствии с протоколом самообнаружения"

[[related_tools]]
agent_name = "skemma"
tool_name = "script_exec"

[[related_tools]]
agent_name = "industrial_iot"
tool_name = "protocol_probe"

[[related_tools]]
agent_name = "remote_operations"
tool_name = "node_discover"

[[related_tools]]
agent_name = "industrial_iot"
tool_name = "device_self_test"

[[related_tools]]
agent_name = "hubris"
tool_name = "report_human"

[features]
location = "cosmos"
execution_mode = "edge"
+++

# Device Interaction SoP

## Standard Operating Procedure for Physical Device Interaction

This skill defines the mandatory interaction protocol when working with physical devices (electrolyzers, sensors, PLCs, etc.) in the Entelecheia system.

### Phase 1: Self-Sensing (Mandatory Before Any Device Access)

**Rule**: Never read/write to a device before completing self-sensing.

1. Call `device_self_test(host=<device_ip>)` to execute the knee-jerk protocol tests
1. Verify `overall_status` is `"online"` or `"degraded"` — if `"offline"`, stop and report to human
1. Review `capability_profile.protocols` to know which protocols the device supports
1. Note `capability_profile.estimated_device_type` for context
1. Check `capability_profile.max_latency_ms` — if >2000ms, warn about slow device

### Phase 2: Safe Data Acquisition

**Rule**: Always prefer batch reads over individual register reads (IEPL batch-first principle).

1. Use `modbus_read()` with batch register ranges to read multiple signals in one call
1. Use `signal_normalize()` for raw value conversion (never trust raw ADC values)
1. Respect device latency — if `max_latency` > 1000ms, add appropriate timeouts
1. For periodic monitoring, establish a consistent scan interval based on device response time

### Phase 3: Safe Device Control (Write Operations)

**Rule**: All writes require human confirmation via `report_human()`.

1. Before any `modbus_write()`, verify the `device_probe` identified the device as writable
1. Signal writes are translated to protocol-native write commands based on the `device_profile`
1. After write, perform a read-back verification on the same signal(s)
1. If read-back doesn't match the written value, report failure immediately
1. Never write to unverified signal addresses — only write to signals identified during device probing

### Emergency Procedures

1. If device becomes unresponsive during interaction, call `device_self_test()` to re-diagnose
1. If device status changes from online to offline mid-operation, stop all writes immediately
1. Critical control writes (safety-related) require double confirmation from human operator
1. All device interactions must be logged through `report_human()` for audit trail

### Device Type Specific Guidelines

| Device Type | Read Strategy | Write Strategy | Safety Threshold |
| --- | --- | --- | --- |
| plc_or_controller | Batch read all exposed signals | Verify signal map before write | Max 1 write per 500ms |
| iot_gateway | Subscribe for real-time, pull for config | Publish with delivery guarantee | Validate payload schema |
| sensor_or_actuator | Periodic poll with signal_normalize | Typically read-only | Respect poll intervals |
| web_service | REST API calls | PUT/POST with schema validation | Rate limit: max 10 req/s |

## Decision Philosophy

- **Self-sensing before access**: Physical devices may be unreachable, degraded, or in an unsafe state. The device self-test is a mandatory gate before any read or write operation. This prevents agents from interacting with devices that are offline or in an error condition.
- **Batch-first reads**: Industrial protocols incur per-transaction latency (often 50-200ms per signal read). Batch reads amortize this overhead. Single-signal reads are only permitted when the signal map is unknown or a single diagnostic value is needed.
- **Human-in-the-loop for writes**: Writing to physical devices changes real-world state (valve positions, pump speeds, heater setpoints). All writes pass through human confirmation via `report_human()`. Critical safety-related writes require double confirmation.
- **Signal normalization as a protocol concern**: Raw ADC values and register readings are meaningless without calibration context. Signal normalization is a first-class protocol operation, not an afterthought, because the physical meaning of a reading is essential for agent decision-making.
- **Physical constraints as first-class concerns**: Devices have real-world constraints — poll interval limits, write rate limits, response latency, and bandwidth caps. The protocol enforces these constraints at the interface level rather than relying on agents to self-regulate.

## Protocol Interface: Device Interaction Protocol (DIP)

All DIP operations are synchronous because device interactions are inherently request-response with bounded latency (typically <200ms per transaction). The physical device may be unreachable, respond slowly, or require specific connection parameters — these are not errors in the protocol but are expected operational states handled through structured result types.

### Operation Signatures

**Device Probing** (via PoleMos):

```text
node_discover(host, port) → {device_profile}
protocol_probe(host, port, protocol) → {protocol_info}
```

**Signal Reading** (sync):

```text
modbus_read(connection_params, register_specs[]) → {readings[]}
```

**Signal Writing** (sync):

```text
modbus_write(connection_params, writes[{register, value}]) → {results[]}
```

**Signal Normalization** (sync):

```text
signal_normalize(raw_value, signal_profile) → {physical_value, unit, uncertainty}
```

**Device Self-Test** (sync):

```text
device_self_test(device_id, test_suite) → {status, capability_profile, latency_ms, errors[]}
```

Executes a self-test protocol on the target device. Returns device status (`"online"`, `"degraded"`, `"offline"`), a capability profile describing what the device can do, measured latency, and any errors encountered. The test suite and result interpretation depend on the device's observed protocol.

### Integration Targets

The DIP is implemented by protocol adapters that translate generic signal operations into protocol-native commands. Rather than hardcoding integrations for specific protocols, the DIP defines what the agent wants (probe the protocol, read signal X, write value Y, normalize the result) and lets the runtime adapter handle the translation:

- **Generic Modbus Adapter**: Uses `node_discover()` for device probing, then `modbus_read()` for register reads (FC03/FC04/FC01/FC02) and `modbus_write()` for register writes (FC06/FC16) with automatic readback verification. Uses `signal_normalize()` for raw-to-physical conversion.
- **Generic OPC UA Adapter** (future): Uses OPC UA Browse/Read/Write service calls for node discovery and read/write operations.
- **Generic MQTT Adapter** (future): Signal reads map to topic subscription with JSON/Sparkplug B payload parsing. Signal writes map to topic publish with configurable QoS.

### Protocol Design Rationale

Physical devices speak whatever protocol their manufacturer chose. The DIP does not encode protocol-specific commands — it describes what we want (discover the protocol, read signal X, write value Y, normalize the result) and lets the runtime adapter translate to whatever protocol the device actually speaks. The protocol is discovered, not assumed.

Industrial devices are physical hardware — PLCs, sensors, actuators — operating in real-world environments with real-world constraints. The DIP abstracts the physical connection (serial/RS-485 or TCP/IP), address space (register map, node namespace, topic tree), and signal normalization behind a standard interface. All DIP operations are synchronous because device interactions are inherently request-response with bounded latency. The protocol treats physical device concerns as first-class concepts: `quality` in read results captures signal integrity (`good`, `uncertain`, `bad`), `timestamp` enables time-series correlation, `uncertainty` in signal normalization communicates measurement confidence, and `readback_value` in writes provides immediate verification that the command reached the physical actuator. Physical devices may be unreachable, respond slowly, or require specific connection parameters — these are not errors in the protocol but are expected operational states handled through structured result types rather than exceptions.
