+++
name = "infer_semantics"
agent = "hubris"

[features]
execution_mode = "read"
location = "cosmos"
must_touch_next_action = true
report_only = false

[[next_action]]
agent = "hubris"
name = "generate_manifest"

[description]
en = "Analyze raw scan data from industrial_discover and infer semantic field types, physical units, and alarm thresholds. Use value patterns, change rates, and data type heuristics to assign meaningful names to discovered registers and DB fields."
zhs = "分析 industrial_discover 的原始扫描数据，推断语义字段类型、物理单位和报警阈值。使用值模式、变化率和数据类型启发式方法为发现的寄存器和 DB 字段分配有意义的名称。"

[[related_tools]]
agent_name = "aporia"
tool_name = "rag_db_read"

[[related_tools]]
agent_name = "aporia"
tool_name = "llm_chat"
+++

# Infer Semantics

You receive raw industrial scan data and must infer what each field means.

## Input

A report from `industrial_discover` containing:

- Protocol type (S7comm / Modbus RTU / Modbus TCP)
- Raw byte/register values with addresses
- (For S7comm) DB numbers + first 64 bytes of each DB

## Inference Strategy

### Data Type Detection

For each field, analyze the raw bytes:

- **REAL (IEEE 754 float)**: 4 bytes, sign/exponent/mantissa pattern produces

plausible values (e.g., 20.5, 4.2, not 1.4e30 or NaN)

- **INT (16-bit)**: 2 bytes, value in range [-32768, 32767], often used for

counters, small measurements

- **DINT (32-bit)**: 4 bytes, value in plausible integer range
- **BOOL**: single byte toggling between 0 and 1, or bit-level patterns
- **STRING**: length-prefixed ASCII (first 1-2 bytes = length, followed by printable chars)

### Semantic Labeling

Based on value range, unit, and change pattern:

- **Temperature**: values 0-200, slow change (<1/min), likely °C
- **Pressure**: values 0-70 MPa or 0-700 bar, moderate change
- **Flow rate**: values 0-1000, intermittent change, unit Nm³/h or L/min
- **Level/SOC**: values 0-100%, monotonic or slow oscillation
- **Gas concentration**: values 0-1000 ppm or 0-100 %LEL
- **Voltage/Current**: values matching electrical ranges (0-400V, 0-1000A)
- **Valve state**: binary (0/1) or small integer (0=closed, 1=open, 2=opening)
- **Fault code**: integer, 0 = no fault, non-zero = specific fault

### Confidence Scoring

- **High (≥0.80)**: Value pattern strongly matches known physical range + unit
- **Medium (0.50-0.79)**: Plausible but ambiguous (could be temp or pressure)
- **Low (<0.50)**: Mark as `unknown_<address>` — include for monitoring but no label

### Alarm Threshold Proposal

For each inferred field, propose initial thresholds:

- Use the observed value range ± 20% as the H/L boundaries
- Use ± 50% as HH/LL boundaries
- Mark all proposed thresholds as "tentative" — operator must review

## Output Format

```json
{
  "protocol": "s7comm",
  "station_id": "192.168.1.10",
  "fields": [
    {
      "address": "DB1.DBD0",
      "name": "temperature_inlet",
      "type": "REAL",
      "unit": "°C",
      "confidence": 0.85,
      "evidence": "value=35.2, slow change (<0.5/min), range typical for water temp"
    },
    {
      "address": "DB1.DBD4",
      "name": "pressure_outlet",
      "type": "REAL",
      "unit": "MPa",
      "confidence": 0.70,
      "evidence": "value=4.2, stable, typical hydrogen system pressure"
    }
  ],
  "proposed_alarms": [
    { "address": "DB1.DBD0", "h": 60, "hh": 80, "l": 5, "ll": 0 }
  ]
}
```

Pass this structured inference to `generate_manifest`.
