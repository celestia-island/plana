+++
name = "generate_manifest"
agent = "hubris"

[features]
execution_mode = "write"
location = "cosmos"
must_touch_next_action = false
report_only = false

[[next_action]]
agent = "classic_software_engineering"
name = "code_verify"

[description]
en = "Generate a HardwareManifest TOML file from the semantic inference results. Write the manifest to disk for operator review and evernight sensor-poll loading."
zh-Hans = "根据语义推断结果生成 HardwareManifest TOML 文件。将清单写入磁盘供操作员审查和 evernight sensor-poll 加载。"

[[related_tools]]
agent_name = "kalos"
tool_name = "file_write"

[[related_tools]]
agent_name = "classic_software_engineering"
tool_name = "code_review"
+++

# Generate Manifest

You receive semantic inference results and must produce a `HardwareManifest`
TOML file that evernight's `sensor-poll --manifest` can load.

## Input

Structured JSON from `infer_semantics` containing protocol, station info,
inferred fields with types/units/confidence, and proposed alarm thresholds.

## Output

A TOML file written to `/workspace/discovered_manifest.toml` following the
`HardwareManifest` schema.

## TOML Template (S7comm example)

```toml
format_version = "1"

[facility]
id = "discovered_corridor"
name = "Auto-Discovered Industrial Corridor"

[[connections]]
id = "conn-s7-1"
kind = "s7comm"
host = "192.168.1.10"
port = 102
rack = 0
slot = 0

[[stations]]
id = 1
connection_ref = "conn-s7-1"
poll_interval_ms = 5000
device_class = "equipment"
vendor = "siemens"

[[stations.s7_data_blocks]]
db_number = 1
start_offset = 0
length = 64

[[stations.s7_data_blocks.fields]]
offset = 0.0
name = "temperature_inlet"
data_type = "REAL"

[stations.s7_data_blocks.fields.scale]
kind = "linear"
factor = 1.0
offset = 0.0
unit = "Celsius"

[stations.s7_data_blocks.fields.alarm]
h = 60.0
hh = 80.0
l = 5.0
ll = 0.0

[[alarm_rules]]
id = "1.temperature_inlet.hh"
station_ref = 1
register_name = "temperature_inlet"
level = "HighHigh"
threshold = 80.0
unit = "°C"
```

## Rules

1. Include ALL discovered fields, even low-confidence ones (mark name as `unknown_<offset>`)
1. Set alarm thresholds from the inference proposal, but round to sensible values
1. Add a `[facility]` with a generic name that the operator can edit later
1. Each discovered station/DB gets its own `[[stations]]` entry
1. Protocol-specific connection goes in `[[connections]]`
1. After writing the file, call `classic_software_engineering.code_verify` to verify the TOML

parses correctly

## Post-Generation

After validation, submit a report to the operator containing:

- File path of the generated manifest
- Summary: N stations, M fields, K alarm rules
- List of fields with confidence < 0.80 (need human review)
- Instructions: "Review the manifest, adjust names/thresholds, then run:

`evernight sensor-poll --manifest discovered_manifest.toml`"
