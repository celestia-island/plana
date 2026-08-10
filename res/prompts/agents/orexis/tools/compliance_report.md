+++
name = "compliance_report"
agent = "orexis"

[description]
en = "Generate a formal compliance audit report for registered standards"
+++

# compliance_report

## Description

Generates a formal compliance audit report based on previously run compliance checks. Can target a specific standard or produce a comprehensive report covering all registered standards. The report includes an executive summary, detailed findings, violation breakdown by severity, and recommendations.

## Parameters

- **`standard_id`** (string, required): The identifier of the standard to generate a report for.
- **`check_results`** (array of objects, required): Array of check result objects from a previous `standard_check` call. Each object should contain `status`, `severity`, `rule_id`, `clause`, `description`, and optionally `deviation`.
- **`device_id`** (string, optional): Device identifier to include in the report.
- **summary** (string, optional): Human-readable summary for stream extraction. Used by the skill-chain reply pipeline.
- **mode** (string, optional): Set to `"reply"` to signal a reply-mode termination (user-facing response).

## Returns

### On Success

```text
Compliance Audit Report

Generated: <ISO 8601 timestamp>
Scope: <scope>
Reporting period: <start> to <end>

Executive Summary:
  Standards evaluated: <number>
  Total rules checked: <number>
  Overall compliance: <percentage>%
  Status: <compliant | partially_compliant | non_compliant>

Breakdown by Standard:

  Standard: <standard_name>
    Status: <compliant | non-compliant>
    Rules: <passed>/<total>
    Critical violations: <number>
    High violations: <number>
    Medium violations: <number>
    Low violations: <number>

    Violations:
      - [<severity>] <rule_id>: <description>
        Expected: <field> <operator> <value>
        Actual: <actual_value>
        Checked at: <timestamp>

  ...

Recommendations:
  1. <recommendation based on violations>
  2. <recommendation based on violations>
  ...
```

### On Failure

```text
Compliance report generation failed

Error: <error message>
```

## Examples

### Example 1: Report for a specific standard

Invocation:

```text
compliance_report
  standard_name: "data_retention_policy"
  scope: "Q1 2024 production audit"
```

Return:

```text
Compliance Audit Report

Generated: 2024-03-10T15:00:00Z
Scope: Q1 2024 production audit
Reporting period: 2024-01-01T00:00:00Z to 2024-03-10T15:00:00Z

Executive Summary:
  Standards evaluated: 1
  Total rules checked: 3
  Overall compliance: 66.7%
  Status: partially_compliant

Breakdown by Standard:

  Standard: data_retention_policy
    Status: non-compliant
    Rules: 2/3
    Critical violations: 1
    High violations: 0
    Medium violations: 0
    Low violations: 0

    Violations:
      - [critical] DR-003: Encryption at rest must be enabled
        Expected: encryption.at_rest eq true
        Actual: false
        Checked at: 2024-03-10T14:30:00Z

Recommendations:
  1. Enable encryption at rest immediately to resolve DR-003 (critical severity)
```

### Example 2: Full audit across all standards

Invocation:

```text
compliance_report
  scope: "full audit"
```

Return:

```text
Compliance Audit Report

Generated: 2024-03-10T16:00:00Z
Scope: full audit
Reporting period: 2024-01-01T00:00:00Z to 2024-03-10T16:00:00Z

Executive Summary:
  Standards evaluated: 2
  Total rules checked: 8
  Overall compliance: 75.0%
  Status: partially_compliant

Breakdown by Standard:

  Standard: data_retention_policy
    Status: non-compliant
    Rules: 2/3
    Critical violations: 1
    ...

  Standard: SOC2
    Status: compliant
    Rules: 4/4
    Critical violations: 0
    ...

Recommendations:
  1. Enable encryption at rest to resolve data_retention_policy DR-003
  2. Continue monitoring SOC2 compliance — all checks passing
```

## Important Notes

- **Data source**: The report aggregates results from previous `standard_check` calls. Run checks before generating a report.
- **Scope field**: The `scope` is a free-text descriptor for organizational purposes and appears in the report header.
- **Empty results**: If no compliance checks have been run against a standard, that standard will show 0 rules checked.
- **Report persistence**: Generated reports are stored and can be retrieved for historical comparison.
- **Recommendations**: Recommendations are auto-generated based on violation patterns. Human review is advised for critical findings.
