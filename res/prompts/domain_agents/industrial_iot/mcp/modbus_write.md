+++
name = "modbus_write"
agent = "industrial_iot"

[description]
en = "Write values to Modbus holding registers"
zh-Hans = "向 Modbus 保持寄存器写入值"
zh-Hant = "向 Modbus 保持暫存器寫入值"
ja = "Modbus 保持レジスタに値を書き込む"
ko = "Modbus 홀딩 레지스터에 값 쓰기"
fr = "Écrire des valeurs dans les registres de maintien Modbus"
es = "Escribir valores en los registros de retención de Modbus"
ru = "Записать значения в удерживающие регистры Modbus"
+++

# modbus_write

Writes one or more values to consecutive holding registers on a Modbus RTU or TCP device. Specify the endpoint, the starting register address, and an array of unsigned 16-bit integer values. Each value corresponds to one register. Supports both single-register and multi-register writes in a single operation.

## Parameters

- **endpoint** (required, string): The Modbus endpoint to connect to. For TCP: `"tcp://host:port"`. For RTU: the serial device path (e.g., `"/dev/ttyUSB0"`).
- **register** (required, number): The starting register address (0-based) to write to.
- **values** (required, array of numbers): An array of unsigned 16-bit integer values to write. Each element maps to one consecutive register.
- **`unit_id`** (optional, number): The Modbus slave unit ID (1–247). Default: `1`.

## Returns

### On Success

Returns `{ ok: true, data: { endpoint: string, register: number, count: number, unit_id: number, status: string }, error: null }`.

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Write a single register

```text
endpoint: "tcp://198.51.100.100:502"
register: 100
values: [1]
```

Returns:

```json
{
  "endpoint": "tcp://198.51.100.100:502",
  "register": 100,
  "count": 1,
  "unit_id": 1,
  "status": "written"
}
```

### Example 2: Write multiple registers

```text
endpoint: "tcp://198.51.100.100:502"
register: 200
values: [100, 200, 300]
unit_id: 5
```

Returns:

```json
{
  "endpoint": "tcp://198.51.100.100:502",
  "register": 200,
  "count": 3,
  "unit_id": 5,
  "status": "written"
}
```

### Example 3: Write to unreachable device

```text
endpoint: "tcp://198.51.100.200:502"
register: 0
values: [42]
```

Returns:

```json
{
  "error": "Connection refused: tcp://198.51.100.200:502"
}
```

## Important Notes

- Register addresses are 0-based (function code 0x10 for multiple, 0x06 for single).
- Values must be unsigned 16-bit integers (0–65535). Out-of-range values will cause an error.
- Writing to incorrect registers on a physical device may cause unexpected behavior. Always verify register mappings before writing.
- This operation is not automatically idempotent — repeated calls will write the same values again.
