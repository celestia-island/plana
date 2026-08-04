+++
name = "modbus_read"
agent = "industrial_iot"

[description]
en = "Read holding registers from a Modbus RTU/TCP device"
zh-Hans = "从 Modbus RTU/TCP 设备读取保持寄存器"
zh-Hant = "從 Modbus RTU/TCP 裝置讀取保持暫存器"
ja = "Modbus RTU/TCP デバイスから保持レジスタを読み取る"
ko = "Modbus RTU/TCP 장치에서 홀딩 레지스터 읽기"
fr = "Lire les registres de maintien d'un appareil Modbus RTU/TCP"
es = "Leer registros de retención de un dispositivo Modbus RTU/TCP"
ru = "Считать удерживающие регистры из устройства Modbus RTU/TCP"
+++

# modbus_read

Reads one or more holding registers from a Modbus RTU or TCP device. Specify the endpoint URL or serial port, the starting register address, and the number of consecutive registers to read. Returns the raw register values as an array of unsigned 16-bit integers. Supports both Modbus TCP (e.g., `tcp://192.168.1.100:502`) and RTU (e.g., `/dev/ttyUSB0`) endpoints.

## Parameters

- **endpoint** (required, string): The Modbus endpoint to connect to. For TCP: `"tcp://host:port"`. For RTU: the serial device path (e.g., `"/dev/ttyUSB0"`).
- **register** (required, number): The starting register address (0-based) to read from.
- **count** (optional, number): The number of consecutive registers to read. Default: `1`.
- **`unit_id`** (optional, number): The Modbus slave unit ID (1–247). Default: `1`.

## Returns

### On Success

Returns `{ ok: true, data: { endpoint: string, register: number, values: [number], count: number, unit_id: number }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Read a single register from a TCP device

```text
endpoint: "tcp://192.168.1.100:502"
register: 100
```

Returns:

```json
{
  "endpoint": "tcp://192.168.1.100:502",
  "register": 100,
  "values": [4231],
  "count": 1,
  "unit_id": 1
}
```

### Example 2: Read multiple registers from an RTU device

```text
endpoint: "/dev/ttyUSB0"
register: 0
count: 10
unit_id: 3
```

Returns:

```json
{
  "endpoint": "/dev/ttyUSB0",
  "register": 0,
  "values": [100, 200, 300, 400, 500, 600, 700, 800, 900, 1000],
  "count": 10,
  "unit_id": 3
}
```

### Example 3: Device not reachable

```text
endpoint: "tcp://192.168.1.200:502"
register: 0
```

Returns:

```json
{
  "error": "Connection refused: tcp://192.168.1.200:502"
}
```

## Important Notes

- Register addresses are 0-based (function code 0x03).
- Values are returned as unsigned 16-bit integers (0–65535). Interpretation (scaling, signed conversion, IEEE 754 float pairs) is the caller's responsibility.
- For RTU endpoints, ensure the serial port is accessible and baud rate/parity settings are pre-configured.
- Connection timeouts may occur if the device is unreachable. Retry logic is the caller's responsibility.
