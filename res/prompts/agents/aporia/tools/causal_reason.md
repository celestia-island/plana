+++
name = "causal_reason"
agent = "aporia"

[description]
en = "Cross-correlation causal reasoning between a target variable and candidate variables"
+++

# causal_reason

## Description

Performs cross-correlation analysis between a target time series and one or more candidate variables to identify potential causal relationships. Computes lagged correlations to detect whether changes in candidate variables precede or follow changes in the target. Returns correlation coefficients at various time lags along with a ranked list of potential causal drivers.

## Parameters

- **target** (string, required): Name of the target variable to analyze.
- **`target_values`** (array of numbers, required): Time series data for the target variable. Must be a numeric array with at least 5 data points.
- **candidates** (array of objects, optional): List of candidate variables. Each object must have `name` (string) and `values` (array of numbers). If omitted, only auto-correlation of the target is computed.

## Returns

### On Success

```text
Causal reasoning complete

Target: <target_name>
Candidates analyzed: <number>
Data points: <number>

Top causal drivers (ranked):
  1. <candidate_name> — max correlation: <score> at lag <n>
  2. <candidate_name> — max correlation: <score> at lag <n>
  ...

Lag analysis for <candidate_name>:
  Lag -3: <correlation>
  Lag -2: <correlation>
  Lag -1: <correlation>
  Lag  0: <correlation>
  Lag +1: <correlation>
  Lag +2: <correlation>
  Lag +3: <correlation>
```

### On Failure

```text
Causal reasoning failed

Error: <error message>
```

## Examples

### Example 1: Analyze CPU load drivers

Invocation:

```text
causal_reason
  target: "response_time_ms"
  target_values: [120, 135, 150, 180, 210, 195, 160, 140, 130, 125]
  candidates:
    - name: "cpu_utilization"
      values: [45, 50, 62, 78, 85, 80, 70, 55, 48, 46]
    - name: "memory_usage"
      values: [60, 61, 62, 63, 65, 64, 63, 62, 61, 60]
```

Return:

```text
Causal reasoning complete

Target: response_time_ms
Candidates analyzed: 2
Data points: 10

Top causal drivers (ranked):
  1. cpu_utilization — max correlation: 0.97 at lag 0
  2. memory_usage — max correlation: 0.85 at lag 1
```

### Example 2: No candidates (auto-correlation only)

Invocation:

```text
causal_reason
  target: "error_rate"
  target_values: [0.01, 0.02, 0.05, 0.08, 0.12, 0.09, 0.04, 0.02, 0.01, 0.01]
```

Return:

```text
Causal reasoning complete

Target: error_rate
Candidates analyzed: 0 (auto-correlation only)
Data points: 10

Auto-correlation peaks:
  Lag 1: 0.89
  Lag 2: 0.65
  Lag 3: 0.31
```

## Important Notes

- **Correlation ≠ causation**: Results indicate statistical association, not guaranteed causality. Domain expertise should validate findings.
- **Data alignment**: All series should be aligned on the same time axis. Mismatched lengths are truncated to the shortest series.
- **Minimum length**: At least 5 data points are required for meaningful correlation analysis.
- **Lag range**: Default lag window is ±(n/3) where n is the series length, capped at ±10.
- **Candidate format**: Each candidate must include both `name` (string) and `values` (numeric array of equal length to `target_values`).
