+++
name = "audit_legality"
agent = "orexis"

[description]
en = "Audit legality compliance against jurisdiction requirements"
+++

# audit_legality

审计目标系统是否符合指定司法管辖区的法律法规要求。

## Parameters

| 参数 | 类型 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- | --- |
| target | string | ✓ | - | 审计目标 |
| jurisdiction | string | | "EU" | 司法管辖区（EU/US/CN/JP等） |

## Returns

```json
{
  "audit_id": "legal-0192a3b4-...",
  "target": "system-001",
  "jurisdiction": "EU",
  "total_requirements": 0,
  "compliant": 0,
  "non_compliant": 0,
  "findings": []
}
```

## Examples

```json
{ "target": "hydrogen-plant-001", "jurisdiction": "EU" }
```
