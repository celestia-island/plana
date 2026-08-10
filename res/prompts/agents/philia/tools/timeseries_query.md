+++
name = "timeseries_query"
agent = "philia"

[description]
en = "Query timeseries data for a metric within a specified time range"
+++

# timeseries_query

## Description

Retrieves timeseries data points for a given metric within an optional time range. If no time range is specified, returns the most recent data points. Results are ordered by timestamp in ascending chronological order.

## Parameters

- **metric** (string, required): The metric name to query (e.g., `"cpu_utilization"`, `"response_time_ms"`).
- **`start_time`** (string, optional): Start of the time range (ISO 8601 format). If omitted, defaults to the earliest available data.
- **`end_time`** (string, optional): End of the time range (ISO 8601 format). If omitted, defaults to the most recent data.

## Returns

### On Success

```text
Timeseries query results

Metric: <metric_name>
Time range: <start> to <end>
Points returned: <number>

  Timestamp                  Value
  2024-03-10T14:00:00Z       45.2
  2024-03-10T14:05:00Z       52.8
  2024-03-10T14:10:00Z       78.1
  ...
```

### No Data

```text
Timeseries query results

Metric: <metric_name>
Time range: <start> to <end>
Points returned: 0

No data points found for the specified metric and time range.
```

### On Failure

```text
Timeseries query failed

Error: <error message>
```

## Examples

### Example 1: Query with time range

Invocation:

```text
timeseries_query
  metric: "cpu_utilization"
  start_time: "2024-03-10T14:00:00Z"
  end_time: "2024-03-10T14:30:00Z"
```

Return:

```text
Timeseries query results

Metric: cpu_utilization
Time range: 2024-03-10T14:00:00Z to 2024-03-10T14:30:00Z
Points returned: 4

  Timestamp                  Value
  2024-03-10T14:00:00Z       45.2
  2024-03-10T14:05:00Z       52.8
  2024-03-10T14:10:00Z       78.1
  2024-03-10T14:15:00Z       65.3
```

### Example 2: Query without time range (recent data)

Invocation:

```text
timeseries_query
  metric: "response_time_ms"
```

Return:

```text
Timeseries query results

Metric: response_time_ms
Time range: all available
Points returned: 10

  Timestamp                  Value
  2024-03-10T13:15:00Z       120.0
  2024-03-10T13:20:00Z       135.5
  ...
```

## Important Notes

- **Metric existence**: Querying a metric that has never been stored returns zero results (not an error).
- **Time range defaults**: Omitting both `start_time` and `end_time` returns all available data for the metric, potentially limited by a system-defined cap.
- **Chronological order**: Results are always returned in ascending timestamp order.
- **Performance**: Narrow time ranges are faster. Avoid querying excessively broad ranges on high-frequency metrics.
