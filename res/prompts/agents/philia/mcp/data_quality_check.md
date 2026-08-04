+++
name = "data_quality_check"
agent = "philia"

[description]
en = "Check data quality for a timeseries metric, detecting gaps, outliers, and staleness"
zh-Hans = "检查时间序列指标的数据质量，检测数据缺失、异常值和过期问题"
zh-Hant = "檢查時間序列指標的資料品質，偵測資料缺失、異常值和過期問題"
ja = "タイムシリーズメトリックのデータ品質をチェックし、ギャップ、外れ値、古さを検出する"
ko = "시계열 메트릭의 데이터 품질을 확인하여 간격, 이상값, 지연을 감지"
fr = "Vérifier la qualité des données d'une métrique de série temporelle, détecter les lacunes, les valeurs aberrantes et l'obsolescence"
es = "Verificar la calidad de los datos de una métrica de series temporales, detectar brechas, valores atípicos y obsolescencia"
ru = "Проверить качество данных метрики временных рядов, обнаружить пропуски, выбросы и устаревание"
+++

# data_quality_check

## Description

Analyzes the stored timeseries data for a given metric and produces a quality report. Checks for data gaps (missing timestamps), statistical outliers, stale data (no recent updates), and basic statistical summaries. Useful for monitoring data pipeline health and detecting ingestion issues.

## Parameters

- **metric** (string, required): The metric name to check data quality for.

## Returns

### On Success

```text
Data quality report

Metric: <metric_name>
Total points: <number>
Time range: <earliest> to <latest>

Quality score: <0–100>

Checks:
  Completeness: <pass | warn | fail> — <details>
  Freshness: <pass | warn | fail> — <details>
  Outliers: <pass | warn | fail> — <details>
  Consistency: <pass | warn | fail> — <details>

Statistics:
  Mean: <value>
  Std deviation: <value>
  Min: <value> at <timestamp>
  Max: <value> at <timestamp>
  Expected interval: <duration>
  Detected gaps: <number>

Issues:
  - <description of issue 1>
  - <description of issue 2>
  ...
```

### No Data

```text
Data quality report

Metric: <metric_name>
Total points: 0

Quality score: N/A

No data available for quality analysis.
```

### On Failure

```text
Data quality check failed

Error: <error message>
```

## Examples

### Example 1: Healthy metric

Invocation:

```text
data_quality_check
  metric: "cpu_utilization"
```

Return:

```text
Data quality report

Metric: cpu_utilization
Total points: 288
Time range: 2024-03-10T00:00:00Z to 2024-03-10T23:55:00Z

Quality score: 95

Checks:
  Completeness: pass — 288/288 expected points present
  Freshness: pass — last data point 2 minutes ago
  Outliers: pass — 0 outliers detected (threshold: 3σ)
  Consistency: pass — regular 5-minute intervals

Statistics:
  Mean: 54.3
  Std deviation: 18.2
  Min: 12.1 at 2024-03-10T03:00:00Z
  Max: 91.7 at 2024-03-10T14:25:00Z
  Expected interval: 5m
  Detected gaps: 0

Issues: none
```

### Example 2: Metric with issues

Invocation:

```text
data_quality_check
  metric: "response_time_ms"
```

Return:

```text
Data quality report

Metric: response_time_ms
Total points: 240
Time range: 2024-03-09T00:00:00Z to 2024-03-09T23:55:00Z

Quality score: 62

Checks:
  Completeness: warn — 240/288 expected points (83%)
  Freshness: fail — last data point 26 hours ago
  Outliers: warn — 5 outliers detected
  Consistency: pass — regular 5-minute intervals (where data exists)

Issues:
  - Gap detected: 2024-03-09T08:00:00Z to 2024-03-09T10:30:00Z (30 missing points)
  - Stale data: no updates in the last 24 hours
  - Outlier at 2024-03-09T14:15:00Z: value=8500ms (expected range: 50–500ms)
```

## Important Notes

- **Quality score**: A composite score from 0–100. Scores above 80 are healthy, 50–80 need attention, below 50 indicate serious data issues.
- **Expected interval**: Automatically inferred from the data. If the metric has irregular spacing, the consistency check may report false positives.
- **Outlier detection**: Uses a 3-standard-deviation threshold by default. Values outside this range are flagged.
- **Freshness threshold**: Data is considered stale if no new points have arrived within 2× the expected interval.
