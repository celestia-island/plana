+++
name = "cpu_info"
agent = "polemos"

[description]
en = "Read and return CPU information from /proc/cpuinfo."
zh-Hans = "读取并返回 /proc/cpuinfo 中的 CPU 信息。"
zh-Hant = "讀取並回傳 /proc/cpuinfo 中的 CPU 資訊。"
ja = "/proc/cpuinfo から CPU 情報を読み取り、返却する。"
ko = "/proc/cpuinfo에서 CPU 정보를 읽어 반환합니다."
fr = "Lire et retourner les informations CPU depuis /proc/cpuinfo."
es = "Leer y devolver información de la CPU desde /proc/cpuinfo."
ru = "Чтение и возврат информации о CPU из /proc/cpuinfo."
+++

# cpu_info

## Description

Reads and returns CPU information from the `/proc/cpuinfo` virtual file on a Linux system. Reports processor architecture, model name, core count, thread count, clock speeds, cache sizes, and supported instruction sets. Useful for capacity planning, compatibility checks, and hardware inventory.

## Parameters

This tool accepts no parameters.

## Returns

### Success

```text
CPU information retrieved

architecture: "x86_64"
model_name: "AMD Ryzen 9 5950X 16-Core Processor"
sockets: 1
cores_per_socket: 16
threads_per_core: 2
total_threads: 32
cpu_mhz_min: 2200.000
cpu_mhz_max: 5081.628
l1d_cache: "512 KiB"
l1i_cache: "512 KiB"
l2_cache: "8 MiB"
l3_cache: "64 MiB"
flags: ["aes", "avx", "avx2", "sse4_1", "sse4_2", "vmx"]
```

### Failure

```text
CPU information retrieval failed

Error: File not found
Message: /proc/cpuinfo is not available. This tool requires a Linux-based operating system.
```

## Examples

### Example 1: Retrieve CPU information

```text
```

## Important Notes

- **Linux-only**: This tool reads from `/proc/cpuinfo`, which is specific to Linux. It is not available on Windows, macOS, or other non-Linux operating systems
- **Virtual environments**: In virtualized or containerized environments, the reported CPU info may reflect the host's processor or a virtualized subset, depending on the hypervisor configuration
- **Static snapshot**: The returned information is a point-in-time snapshot. CPU frequencies may change dynamically due to power management
- **Permissions**: Reading `/proc/cpuinfo` does not require elevated privileges on most Linux distributions
