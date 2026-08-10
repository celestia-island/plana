+++
name = "modbus_read"
agent = "industrial_iot"

[description]
en = "Read holding registers from a Modbus RTU/TCP device"
+++

# modbus_read

Reads one or more holding registers from a Modbus RTU or TCP device. Specify the endpoint URL or serial port, the starting register address, and the number of consecutive registers to read. Returns the raw register values as an array of unsigned 16-bit integers. Supports both Modbus TCP (e.g., `tcp://198.51.100.100:502`) and RTU (e.g., `/dev/ttyUSB0`) endpoints.

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
endpoint: "tcp://198.51.100.100:502"
register: 100
```

Returns:

```json
{
  "endpoint": "tcp://198.51.100.100:502",
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
endpoint: "tcp://198.51.100.200:502"
register: 0
```

Returns:

```json
{
  "error": "Connection refused: tcp://198.51.100.200:502"
}
```

## Important Notes

- Register addresses are 0-based (function code 0x03).
- Values are returned as unsigned 16-bit integers (0–65535). Interpretation (scaling, signed conversion, IEEE 754 float pairs) is the caller's responsibility.
- For RTU endpoints, ensure the serial port is accessible and baud rate/parity settings are pre-configured.
- Connection timeouts may occur if the device is unreachable. Retry logic is the caller's responsibility.
