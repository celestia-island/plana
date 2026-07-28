+++
name = "manage_sensitivity_rules"
agent = "orexis"

[description]
en = "Get or update sensitivity redaction rules"
zhs = "获取或更新敏感数据脱敏规则"
zht = "獲取或更新敏感數據脫敏規則"
ja = "機密データのマスキングルールを取得または更新"
ko = "민감 데이터 마스킹 규칙 조회 또는 업데이트"
fr = "Obtenir ou mettre à jour les règles de masquage de données sensibles"
es = "Obtener o actualizar reglas de redacción de datos sensibles"
ru = "Получить или обновить правила маскирования конфиденциальных данных"
+++

# manage_sensitivity_rules

Manages sensitivity redaction rules in the OreXis security policy store. Supports two actions: `get_rules` returns the current field-level redaction rules and command allowlist; `update_rules` replaces the entire rule set with a new one.

## Parameters

| Name | Type | Required | Default | Description |
| --- | --- | --- | --- | --- |
| `action` | string | yes | — | `"get_rules"` or `"update_rules"` |
| `rules` | object | conditional | — | Required when `action = "update_rules"`. A `SensitivityRuleSet` with `field_rules` (array) and `command_allowlist` (array) |

## Returns

### On `get_rules` Success

```json
{
  "field_rules": [],
  "command_allowlist": []
}
```

### On `update_rules` Success

```json
{
  "updated": true,
  "field_rule_count": 12,
  "command_rule_count": 5
}
```

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Get current rules

```json
{ "action": "get_rules" }
```

### Example 2: Update rules

```json
{
  "action": "update_rules",
  "rules": {
    "field_rules": [{"field": "password", "action": "redact"}],
    "command_allowlist": ["ls", "cat"]
  }
}
```

## Important Notes

- `update_rules` replaces the entire rule set atomically. Both `field_rules` and `command_allowlist` must be non-empty — the system rejects empty arrays to preserve safety.
- The sensitivity policy store must be initialized before calling this tool.
