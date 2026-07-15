+++
name = "protocol_auto_detect"
agent = "industrial_iot"

[description]
en = "Auto-detect the protocol of an unknown transport endpoint (TCP or serial) via a probe chain."
zhs = "通过探测链自动识别未知传输端点（TCP 或串口）使用的协议。"
+++

# protocol_auto_detect

## Description

Single entry point for protocol identification during autonomous corridor
discovery. Given a transport endpoint, runs evernight's protocol probe chain
and returns the identified protocol with a confidence score. The
`industrial_discover` coordinator uses this result to decide which deep-scan
tool to dispatch next (`s7comm_discover`, `serial_discover`, `device_self_test`).

This tool is **read-only**: it never writes to a device.

## Parameters

- **transport** (string, required): `"tcp"` or `"serial"`.
- **host** (string, tcp): IP address or hostname.
- **port** (number, tcp, optional): TCP port. When omitted, probes the common

industrial ports 102, 502, 4840, 1883, 80, 8080.

- **`serial_port`** (string, serial): device path (e.g. `"/dev/ttyUSB0"`).
- **baud** (number, serial, optional): baud-rate hint; if omitted the probe

sweeps standard rates (1200–115200).

- **`station_id`** (number, serial, optional): Modbus station id used during the

baud sweep. Default: `1`.

## Returns

- `detected` (bool): whether a known protocol was identified.
- `protocol` (string): `"s7comm"`, `"modbus-tcp"`, `"opc-ua"`, `"mqtt"`,

`"http"`, `"modbus-rtu"`, or `"unknown"`.

- `confidence` (number): 0.0–1.0.
- `banner` (string?, optional): handshake detail.
- `transport` (object): echoed transport descriptor (kind, host/port or

`serial_port`/baud/`station_id`).
