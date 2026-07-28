+++
name = "s7comm_discover"
agent = "industrial_iot"

[description]
en = "Connect to a Siemens S7 PLC over TCP:102, scan data blocks, and probe DB structure for type inference."
zh-Hans = "通过 TCP:102 连接西门子 S7 PLC，扫描数据块并探测 DB 结构以供类型推断。"
+++

# s7comm_discover

## Description

Connects to an S7comm (Siemens) PLC, negotiates the PDU, optionally scans a
range of DB numbers, and probes the first bytes of each readable DB. The raw
byte dumps feed the `infer_semantics` skill for LLM-based field-type inference.

This tool is **read-only**: it never writes to a device.

## Parameters

- **host** (string, required): PLC IP address.
- **port** (number, optional): TCP port. Default: `102`.
- **rack** (number, optional): rack number. Default: `0`.
- **slot** (number, optional): slot number. Default: `0` (S7-1200/1500), `2` for S7-300.
- **`scan_dbs`** (bool, optional): scan DB numbers. Default: `true`.
- **`db_start`** (number, optional): first DB number. Default: `1`.
- **`db_end`** (number, optional): last DB number. Default: `100`.
- **`probe_structure`** (bool, optional): read first 64 bytes of each found DB. Default: `true`.

## Returns

- `connected` (bool), `pdu_length` (number).
- `dbs` (array): per-DB `{ db_number, status, readable_bytes, raw_bytes_hex?, raw_byte_count? }`.
- `db_scan_error` (string?, optional): present when the DB scan failed.
