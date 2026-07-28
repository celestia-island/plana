+++
name = "verify_write_safety"
agent = "orexis"

[description]
en = "Pre-write safety verification: confirms the address is whitelisted and the value is within approved bounds."
zhs = "写入前安全校验：确认地址已加入白名单且数值在批准范围内。"
+++

# verify_write_safety

## Description

Pre-write gate for industrial writes. Performs checks (1) the address is on
the whitelist and (2) the proposed value is within the approved bounds. The
caller must still perform read-after-write confirmation once the write returns.

## Parameters

- **`station_id`** (string, required): station identifier.
- **protocol** (string): industrial protocol of the write.
- **address** (string, required): target address.
- **value** (number, required): proposed value to write.

## Returns

- `allowed` (bool): whether the write may proceed.
- `outcome` (string): `allowed` | `default_deny_active` |

`address_not_whitelisted` | `value_out_of_bounds`.

- `detail` (string): human-readable explanation.
