+++
name = "audit_alignment"
agent = "orexis"

[description]
en = "Audit alignment of operations against registered standards"
+++

# audit_alignment

审计操作与已注册标准规则的对齐情况。

## Parameters

| 参数 | 类型 | 必填 | 说明 |
| --- | --- | --- | --- |
| target | string | ✓ | 审计目标（设备ID、系统名称等） |

## Returns

```json
{
  "audit_id": "align-0192a3b4-...",
  "target": "device-001",
  "total_rules": 0,
  "passed": 0,
  "failed": 0,
  "findings": []
}
```

## Examples

```json
{ "target": "device-001" }
```

## Notes

- 需先通过 `standard_register` 注册标准规则
- 审计结果包含通过/失败计数和详细发现
