+++
name = "set_default_write_policy"
agent = "orexis"

[description]
en = "Toggle the default-deny industrial write policy for unknown hardware."
+++

# set_default_write_policy

## Description

When `active=true`, every industrial write (Modbus/S7comm/MC) to a
station/address that is **not** on the whitelist is denied by default. This is
the safe state immediately after autonomous discovery of an unknown corridor.
Operators lift the restriction per-address via `whitelist_write_address`.

## Parameters

- **active** (bool, required): `true` = default-deny, `false` = permissive.

## Returns

- `default_deny` (bool): new policy state.
- `updated` (bool): always `true` on success.
