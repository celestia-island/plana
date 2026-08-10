+++
name = "request_write_approval"
agent = "orexis"

[description]
en = "Request operator approval for a safety-critical industrial write. Suspends until the operator responds (default 120s timeout)."
+++

# request_write_approval

## Description

Human-in-the-loop approval gate for industrial writes (Phase D.2). The agent
calls this when [`verify_write_safety`](./verify_write_safety.md) returns
`Denied`. This tool builds a `WriteApprovalRequest`, broadcasts it to the
operator (rendered as an approval dialog in shittim-chest via
`TuiMessage::IndustrialWriteApprovalPush`), then **suspends** until the operator
responds or the timeout expires.

On approval the operator's response also adds a temporary whitelist entry
(station/address via `industrial.approveWrite`), so the agent's subsequent write
passes `verify_write_safety` without a second approval round. The caller should
then execute the write and perform read-back verification.

On denial or timeout the caller should adjust its plan accordingly.

## Parameters

- **`station_id`** (string, required): station identifier.
- **protocol** (string): industrial protocol (`modbus_rtu`, `s7comm`).
- **address** (string, required): target register / DB offset address.
- **`field_name`** (string): human-readable field name being written.
- **`current_value`** (number): current value at the address (for the operator's reference).
- **`proposed_value`** (number, required): value the agent wants to write.
- **unit** (string): engineering unit (e.g. MPa, °C).
- **reason** (string): why the agent wants to perform this write.
- **`timeout_secs`** (integer): seconds to wait for operator response (default 120).

## Returns

- `approved` (bool): whether the operator approved the write.
- `request_id` (string): unique id for this approval request.
- `timed_out` (bool): whether the request expired before the operator responded.
- `reason` (string): operator's comment, or timeout/error explanation.
- `approved_by` (string): who responded.
