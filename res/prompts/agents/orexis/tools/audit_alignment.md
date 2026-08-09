+++
name = "audit_alignment"
agent = "orexis"

[description]
en = "Audit alignment of operations against registered standards"
zh-Hans = "审计操作与注册标准的对齐情况"
zh-Hant = "審計操作與註冊標準的對齊情況"
ja = "登録基準に対する運用の整合性を監査"
ko = "등록된 표준에 대한 운영 정렬 감사"
fr = "Auditer l'alignement des opérations par rapport aux normes enregistrées"
es = "Auditar la alineación de operaciones con los estándares registrados"
ru = "Аудит соответствия операций зарегистрированным стандартам"
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
