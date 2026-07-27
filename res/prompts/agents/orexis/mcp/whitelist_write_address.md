+++
name = "whitelist_write_address"
agent = "orexis"

[description]
en = "Approve (or revoke) write access for a specific register/DB offset on a station, with optional value bounds."
zhs = "批准（或撤销）某站点上特定寄存器/DB 偏移的写入权限，可设置数值上下限。"
+++

# whitelist_write_address

## Description

Explicitly approves an industrial write target so that `verify_write_safety`
(and the default-deny policy) will permit writes to it. Optionally constrains
the approved value range and, for Modbus, a contiguous register block.

## Parameters

- **`station_id`** (string, required): station identifier.
- **protocol** (string): `modbus_rtu` | `modbus_tcp` | `s7comm` | `mc_protocol`.
- **address** (string): address label, e.g. `DB1.DBD0` / `HR:40001` / `valve_open`.
- **`register_start`** (int, optional): Modbus register range start.
- **`register_count`** (int, optional): Modbus register block length.
- **`min_value`** (number, optional): approved minimum (inclusive).
- **`max_value`** (number, optional): approved maximum (inclusive).
- **remove** (bool, optional): if `true`, revoke matching approvals.
- **`approved_by`** (string, optional): who approved (default: `operator`).
- **reason** (string, optional): reason / ticket reference.

## Returns

- `added` (bool) or `removed` (bool).
- `total_entries` / `entries_removed`.
