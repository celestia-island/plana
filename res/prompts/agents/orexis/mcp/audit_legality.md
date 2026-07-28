+++
name = "audit_legality"
agent = "orexis"

[description]
en = "Audit legality compliance against jurisdiction requirements"
zhs = "审计合法性与司法管辖区要求的合规性"
zht = "審計合法性與司法管轄區要求的合規性"
ja = "管轄区要件に対する適法性コンプライアンスを監査"
ko = "관할 구역 요구 사항에 대한 합법성 규정 준수 감사"
fr = "Auditer la conformité légale par rapport aux exigences de juridiction"
es = "Auditar el cumplimiento legal frente a los requisitos jurisdiccionales"
ru = "Аудит правового соответствия требованиям юрисдикции"
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
