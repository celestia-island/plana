+++
name = "memory_info"
agent = "polemos"

[description]
en = "Read and return memory statistics from /proc/meminfo."
zh-Hans = "读取并返回 /proc/meminfo 中的内存统计信息。"
zh-Hant = "讀取並回傳 /proc/meminfo 中的記憶體統計資訊。"
ja = "/proc/meminfo からメモリ統計情報を読み取り、返却する。"
ko = "/proc/meminfo에서 메모리 통계 정보를 읽어 반환합니다."
fr = "Lire et retourner les statistiques mémoire depuis /proc/meminfo."
es = "Leer y devolver estadísticas de memoria desde /proc/meminfo."
ru = "Чтение и возврат статистики памяти из /proc/meminfo."
+++

# memory_info

## Description

Reads and returns memory statistics from the `/proc/meminfo` virtual file on a Linux system. Reports total, free, and available physical memory along with swap usage, buffer/cache allocations, and other kernel memory metrics. Useful for monitoring resource utilization, diagnosing memory pressure, and capacity planning.

## Parameters

This tool accepts no parameters.

## Returns

### Success

```text
Memory information retrieved

mem_total_kib: 32768000
mem_free_kib: 8245600
mem_available_kib: 18432000
buffers_kib: 1024000
cached_kib: 9216000
swap_total_kib: 8192000
swap_free_kib: 7987000
swap_cached_kib: 45000
hugepages_total: 0
hugepages_free: 0
```

### Failure

```text
Memory information retrieval failed

Error: File not found
Message: /proc/meminfo is not available. This tool requires a Linux-based operating system.
```

## Examples

### Example 1: Retrieve memory information

```text
```

## Important Notes

- **Linux-only**: This tool reads from `/proc/meminfo`, which is specific to Linux. It is not available on Windows, macOS, or other non-Linux operating systems
- **Available vs. free**: `mem_available` is a more accurate indicator of usable memory than `mem_free`, as it accounts for reclaimable buffers and caches. Prefer `mem_available` for capacity decisions
- **KiB units**: All values are reported in kibibytes (KiB). Divide by 1024 to convert to mebibytes (MiB) or by 1048576 for gibibytes (GiB)
- **Dynamic values**: Memory statistics change rapidly. The returned values represent a point-in-time snapshot and may differ on subsequent calls
- **Permissions**: Reading `/proc/meminfo` does not require elevated privileges on most Linux distributions
