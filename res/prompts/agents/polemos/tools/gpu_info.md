+++
name = "gpu_info"
agent = "polemos"

[description]
en = "Query nvidia-smi for GPU information (requires NVIDIA drivers)."
+++

# gpu_info

## Description

Queries `nvidia-smi` for GPU information on systems equipped with NVIDIA graphics cards. Reports GPU model, driver version, CUDA version, memory allocation, utilization, temperature, power usage, and running processes per GPU. Useful for monitoring GPU health, resource allocation, and diagnosing performance issues in GPU-accelerated workloads.

## Parameters

This tool accepts no parameters.

## Returns

### Success

```text
GPU information retrieved

driver_version: "535.129.03"
cuda_version: "12.2"

gpu_index: 0
  name: "NVIDIA GeForce RTX 4090"
  uuid: "GPU-e56a1c9d-2f3b-4a8e-9c1d-7b3e4f5a6d8c"
  temperature_c: 42
  utilization_gpu_percent: 35
  utilization_memory_percent: 28
  memory_total_mib: 24564
  memory_used_mib: 6878
  memory_free_mib: 17686
  power_draw_w: 120.5
  power_limit_w: 450.0
  processes:
    - pid: 12345
      name: "python3"
      memory_used_mib: 4096

gpu_index: 1
  name: "NVIDIA GeForce RTX 4090"
  uuid: "GPU-a1b2c3d4-5e6f-7a8b-9c0d-1e2f3a4b5c6d"
  temperature_c: 38
  utilization_gpu_percent: 0
  utilization_memory_percent: 0
  memory_total_mib: 24564
  memory_used_mib: 0
  memory_free_mib: 24564
  power_draw_w: 25.3
  power_limit_w: 450.0
  processes: []
```

### Failure

```text
GPU information retrieval failed

Error: Command not found
Message: nvidia-smi is not available. Ensure NVIDIA drivers are installed and nvidia-smi is in the system PATH.
```

## Examples

### Example 1: Retrieve GPU information

```text
```

## Important Notes

- **NVIDIA-only**: This tool exclusively supports NVIDIA GPUs. AMD and Intel GPUs are not supported by `nvidia-smi`
- **Driver dependency**: The NVIDIA driver and CUDA toolkit must be properly installed. Without the driver, `nvidia-smi` will not be available
- **Multi-GPU systems**: The tool reports information for all detected NVIDIA GPUs. Each GPU is listed with its index and unique UUID
- **Permissions**: In most configurations, `nvidia-smi` does not require root privileges. However, some restricted environments may limit access to GPU metrics
- **Overhead**: Querying `nvidia-smi` introduces minimal overhead. However, avoid polling at very high frequencies (sub-second) in production environments
- **Memory units**: All memory values are reported in mebibytes (MiB). Divide by 1024 to convert to gibibytes (GiB)
