+++
name = "device_self_test"
agent = "industrial_iot"

[description]
en = "Run hardware diagnostic self-tests on a connected device or the local host."
zhs = "在连接的设备或本地主机上运行硬件诊断自检。"
zht = "在連接的裝置或本機主機上執行硬體診斷自我測試。"
ja = "接続されたデバイスまたはローカルホストでハードウェア診断セルフテストを実行する。"
ko = "연결된 장치 또는 로컬 호스트에서 하드웨어 진단 자체 테스트를 실행합니다."
fr = "Exécuter des autotests de diagnostic matériel sur un appareil connecté ou l'hôte local."
es = "Ejecutar autopruebas de diagnóstico de hardware en un dispositivo conectado o el host local."
ru = "Запуск аппаратной диагностики (самотестирования) на подключённом устройстве или локальном хосте."
+++

# device_self_test

## Description

Runs hardware diagnostic self-tests on a connected device or the local host. Supports multiple test scopes including quick health checks, full comprehensive diagnostics, memory-specific tests, and disk-specific tests. Reports overall status along with per-component results. Useful for validating node health before deployment or troubleshooting hardware issues.

## Parameters

- **`test_type`** (string, optional): Scope of the diagnostic test to perform. Accepted values: `"quick"` — fast health check of major subsystems; `"full"` — comprehensive diagnostics covering all hardware; `"memory"` — focused memory (RAM) test; `"disk"` — focused storage/disk test. Default: `"quick"`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Device self-test completed

test_type: "quick"
overall_status: "pass"

component: "cpu"
status: "pass"
message: "All cores responsive, no thermal throttling"

component: "memory"
status: "pass"
message: "16 GB available, 0 errors detected"

component: "disk"
status: "pass"
message: "SMART status healthy, no bad sectors"

component: "network"
status: "pass"
message: "All interfaces operational"
```

### Failure

```text
Device self-test completed

test_type: "full"
overall_status: "fail"

component: "cpu"
status: "pass"
message: "All cores responsive"

component: "memory"
status: "fail"
message: "ECC errors detected on DIMM slot A2"

component: "disk"
status: "warn"
message: "SMART indicates 3 reallocated sectors on /dev/sda"

component: "network"
status: "pass"
message: "All interfaces operational"
```

## Examples

### Example 1: Quick health check

```text
test_type: "quick"
```

### Example 2: Full comprehensive diagnostics

```text
test_type: "full"
```

### Example 3: Memory-only test

```text
test_type: "memory"
```

### Example 4: Disk-only test

```text
test_type: "disk"
```

## Important Notes

- **Test duration**: `"quick"` tests typically complete in seconds. `"full"` and `"memory"` tests may take several minutes depending on system resources. Schedule accordingly to avoid service disruption
- **Destructive potential**: Some `"full"` memory tests are intensive and may destabilize an already faulty system. Run during maintenance windows when possible
- **SMART dependency**: Disk tests rely on S.M.A.R.T. data being available. Ensure `smartmontools` is installed on the target node
- **Root privileges**: Certain test types (especially `"disk"` and `"memory"`) may require elevated privileges to access low-level hardware interfaces
