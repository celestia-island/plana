+++
name = "signal_normalize"
agent = "skemma"

[description]
en = "Apply normalization transforms to signal data"
zhs = "对信号数据应用归一化变换"
zht = "對訊號資料應用歸一化變換"
ja = "信号データに正規化変換を適用"
ko = "신호 데이터에 정규화 변환 적용"
fr = "Appliquer des transforms de normalisation aux données de signal"
es = "Aplicar transformaciones de normalización a datos de señal"
ru = "Применить преобразования нормализации к данным сигнала"
+++

# signal_normalize

Applies a normalization transform to an array of numeric signal values and returns the normalized result along with the transform parameters used. Three methods are supported: min-max scaling (scales to [0, 1]), z-score standardization (zero mean, unit variance), and decimal scaling (divides by a power of 10). The returned metadata includes the parameters needed to reverse the transform.

## Parameters

- **values** (required, array of numbers): The input signal data to normalize. Must contain at least one numeric element.
- **method** (optional, string): The normalization method to apply. Options: `"minmax"`, `"zscore"`, `"decimal"`. Default: `"minmax"`.

## Returns

### On Success

Returns `{ ok: true, data: { method: string, input_count: number, normalized: [number], parameters: object }, error: null }`.

The `parameters` object varies by method:

- **minmax**: `{ min: number, max: number }`
- **zscore**: `{ mean: number, std: number }`
- **decimal**: `{ scale: number }` (the power-of-10 divisor)

### On Failure

Returns `{ ok: false, data: null, error: string }`.

## Examples

### Example 1: Min-max normalization (default)

```text
values: [10, 20, 30, 40, 50]
```

Returns:

```json
{
  "method": "minmax",
  "input_count": 5,
  "normalized": [0.0, 0.25, 0.5, 0.75, 1.0],
  "parameters": {
    "min": 10,
    "max": 50
  }
}
```

### Example 2: Z-score standardization

```text
values: [2, 4, 4, 4, 5, 5, 7, 9]
method: "zscore"
```

Returns:

```json
{
  "method": "zscore",
  "input_count": 8,
  "normalized": [-1.5, -0.5, -0.5, -0.5, 0.0, 0.0, 1.0, 2.0],
  "parameters": {
    "mean": 5.0,
    "std": 2.0
  }
}
```

### Example 3: Decimal scaling

```text
values: [123, -456, 789]
method: "decimal"
```

Returns:

```json
{
  "method": "decimal",
  "input_count": 3,
  "normalized": [0.123, -0.456, 0.789],
  "parameters": {
    "scale": 1000
  }
}
```

## Important Notes

- **Min-max**: If all input values are identical, the result is `[0.5]` for each element (division by zero is avoided).
- **Z-score**: If standard deviation is zero (constant signal), normalized values are all `0.0`.
- **Decimal scaling**: The scale factor is `10^d` where `d` is the number of digits in the maximum absolute value.
- The `parameters` object enables exact inverse transformation of the normalized values.
- Input arrays with `NaN` or `Infinity` values will result in an error.
