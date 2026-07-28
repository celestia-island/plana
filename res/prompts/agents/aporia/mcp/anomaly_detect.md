+++
name = "anomaly_detect"
agent = "aporia"

[description]
en = "Detect anomalies in numeric data using isolation forest, z-score, or IQR methods"
zhs = "使用隔离森林、Z分数或IQR方法检测数值数据中的异常"
zht = "使用隔離森林、Z分數或IQR方法檢測數值資料中的異常"
ja = "アイソレーションフォレスト、Zスコア、またはIQRメソッドを使用して数値データの異常を検出する"
ko = "격리 포리스트, Z-점수 또는 IQR 방법을 사용하여 수치 데이터의 이상 감지"
fr = "Détecter les anomalies dans les données numériques via isolation forest, z-score ou IQR"
es = "Detectar anomalías en datos numéricos mediante isolation forest, z-score o IQR"
ru = "Обнаружение аномалий в числовых данных методами isolation forest, z-score или IQR"
+++

# anomaly_detect

## Description

Analyzes a series of numeric values to identify outliers and anomalies. Supports three detection methods: **Z-score** (standard deviation based), **IQR** (interquartile range based), and **Isolation Forest** (tree-based ensemble). The tool returns the indices and values of detected anomalies along with summary statistics.

## Parameters

- **values** (array of numbers, required): The numeric data series to analyze. Must contain at least 3 data points.
- **method** (string, optional): Detection algorithm to use. One of `"zscore"`, `"iqr"`, `"isolation"`. Defaults to `"zscore"`.

## Returns

### On Success

```text
Anomaly detection complete

Method: <method>
Total points: <number>
Anomalies found: <number>

Summary statistics:
  Mean: <value>
  Std deviation: <value>
  Min: <value>
  Max: <value>

Anomalies:
  Index <i>: value=<v>, score=<s>
  Index <j>: value=<v>, score=<s>
  ...
```

### On Failure

```text
Anomaly detection failed

Error: <error message>
```

## Examples

### Example 1: Z-score detection

Invocation:

```text
anomaly_detect
  values: [10, 12, 11, 13, 10, 100, 12, 11, 14, 9]
  method: "zscore"
```

Return:

```text
Anomaly detection complete

Method: zscore
Total points: 10
Anomalies found: 1

Summary statistics:
  Mean: 20.2
  Std deviation: 27.1
  Min: 9
  Max: 100

Anomalies:
  Index 5: value=100, score=3.47
```

### Example 2: IQR method

Invocation:

```text
anomaly_detect
  values: [22, 24, 21, 23, 25, 22, 60, 24, 23, 22]
  method: "iqr"
```

Return:

```text
Anomaly detection complete

Method: iqr
Total points: 10
Anomalies found: 1

Anomalies:
  Index 6: value=60, bounds=[17.5, 29.5]
```

## Important Notes

- **Minimum data points**: At least 3 values are required. Fewer inputs will result in an error.
- **Method selection**: Use `zscore` for normally distributed data, `iqr` for skewed distributions, and `isolation` for high-dimensional or multi-modal data.
- **Score interpretation**: Z-score values above 2–3 typically indicate anomalies. IQR flags points outside 1.5× IQR from quartiles.
- **Isolation Forest**: The isolation method may produce non-deterministic results due to random partitioning.
