+++
name = "standard_check"
agent = "orexis"

[description]
en = "Check data against registered compliance standard rules"
+++

# standard_check

## Description

Evaluates a data object against a previously registered compliance standard. Each rule in the standard is checked against the provided data, producing a pass/fail result with severity classification for any violations. Returns a detailed compliance report suitable for audit trails.

## Parameters

- **`standard_id`** (string, required): The UUID of the registered compliance standard to check against. Also accepts `standard_name` as a human-readable alias.
- **data** (object, required): The data object to evaluate. Fields in this object are matched against the `field` paths defined in the standard's rules.

## Returns

### On Success

```text
Compliance check complete

Standard: <standard_name>
Data fields evaluated: <number>
Rules checked: <number>
Passed: <number>
Failed: <number>
Overall status: <compliant | non-compliant>

Results:
  Rule <rule_id>: <PASS | FAIL> — <description>
    Field: <field_path>
    Expected: <operator> <value>
    Actual: <actual_value>
    Severity: <severity> (for failures only)

  ...

Violations:
  - [<severity>] <rule_id>: <description>
    Expected: <field_path> <operator> <value>
    Actual: <actual_value>
  ...
```

### On Failure

```text
Compliance check failed

Error: Standard '<standard_name>' not found. Register it with standard_register first.
```

## Examples

### Example 1: Fully compliant data

Invocation:

```text
standard_check
  standard_name: "data_retention_policy"
  data:
    access_logs:
      retention_days: 120
    pii:
      purge_days: 300
    encryption:
      at_rest: true
```

Return:

```text
Compliance check complete

Standard: data_retention_policy
Data fields evaluated: 3
Rules checked: 3
Passed: 3
Failed: 0
Overall status: compliant

Results:
  Rule DR-001: PASS — Access logs must be retained for at least 90 days
    Field: access_logs.retention_days
    Expected: gte 90
    Actual: 120

  Rule DR-002: PASS — PII data must be purged within 365 days
    Field: pii.purge_days
    Expected: lte 365
    Actual: 300

  Rule DR-003: PASS — Encryption at rest must be enabled
    Field: encryption.at_rest
    Expected: eq true
    Actual: true

Violations: none
```

### Example 2: Non-compliant data

Invocation:

```text
standard_check
  standard_name: "data_retention_policy"
  data:
    access_logs:
      retention_days: 30
    pii:
      purge_days: 500
    encryption:
      at_rest: false
```

Return:

```text
Compliance check complete

Standard: data_retention_policy
Data fields evaluated: 3
Rules checked: 3
Passed: 0
Failed: 3
Overall status: non-compliant

Violations:
  - [critical] DR-001: Access logs must be retained for at least 90 days
    Expected: access_logs.retention_days gte 90
    Actual: 30

  - [high] DR-002: PII data must be purged within 365 days
    Expected: pii.purge_days lte 365
    Actual: 500

  - [critical] DR-003: Encryption at rest must be enabled
    Expected: encryption.at_rest eq true
    Actual: false
```

## Important Notes

- **Prerequisite**: The standard must be registered via `standard_register` before it can be checked.
- **Missing fields**: If a rule references a field not present in the data, the check fails with severity `critical`.
- **Nested data**: Use dot-notation in rule field paths to access nested properties (e.g., `"security.encryption.enabled"` resolves `data.security.encryption.enabled`).
- **Immutable results**: Check results are stored for audit trail purposes and can be referenced in `compliance_report`.
