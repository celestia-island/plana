+++
name = "storage_info"
agent = "polemos"

[description]
en = "Read disk and partition information from the system."
+++

# storage_info

## Description

Reads disk and partition information from the system by collecting data from `lsblk`, `df`, and related system utilities. Reports block devices, partitions, mount points, filesystem types, total and used capacity, and SMART health status where available. Useful for storage inventory, capacity monitoring, and disk health assessment.

## Parameters

This tool accepts no parameters.

## Returns

### Success

```text
Storage information retrieved

device: "/dev/sda"
size: "500 GB"
type: "disk"
model: "Samsung SSD 860"
serial: "S3Z8NB0K123456"
partitions:
  - partition: "/dev/sda1"
    size: "512 MB"
    fstype: "vfat"
    mountpoint: "/boot/efi"
    usage_total: "512 MB"
    usage_used: "6 MB"
    usage_percent: "1%"
  - partition: "/dev/sda2"
    size: "499.5 GB"
    fstype: "ext4"
    mountpoint: "/"
    usage_total: "499.5 GB"
    usage_used: "187.3 GB"
    usage_percent: "37%"

device: "/dev/sdb"
size: "2 TB"
type: "disk"
model: "WDC WD20EZRZ"
partitions:
  - partition: "/dev/sdb1"
    size: "2 TB"
    fstype: "ext4"
    mountpoint: "/data"
    usage_total: "2 TB"
    usage_used: "1.2 TB"
    usage_percent: "60%"
```

### Failure

```text
Storage information retrieval failed

Error: Command failed
Message: Unable to execute lsblk. Ensure util-linux is installed on the target system.
```

## Examples

### Example 1: Retrieve storage information

```text
```

## Important Notes

- **Linux-only**: This tool relies on Linux-specific utilities (`lsblk`, `df`). It is not available on Windows or macOS
- **Dependency**: Requires `util-linux` package (provides `lsblk`) and `smartmontools` (optional, for SMART data). Most Linux distributions include `util-linux` by default
- **Mount details**: Unmounted partitions will report `mountpoint` as empty. Use `usage_*` fields only for mounted filesystems
- **Permissions**: Basic storage listing does not require root. However, SMART health data and serial numbers may require elevated privileges
- **Dynamic data**: Storage usage changes continuously. Returned values are a point-in-time snapshot
