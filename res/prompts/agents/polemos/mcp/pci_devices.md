+++
name = "pci_devices"
agent = "polemos"

[description]
en = "List PCI devices from lspci output."
zhs = "从 lspci 输出中列出 PCI 设备。"
zht = "從 lspci 輸出中列出 PCI 裝置。"
ja = "lspci 出力から PCI デバイスを一覧表示する。"
ko = "lspci 출력에서 PCI 장치를 나열합니다."
fr = "Lister les périphériques PCI depuis la sortie de lspci."
es = "Listar dispositivos PCI desde la salida de lspci."
ru = "Список PCI-устройств из вывода lspci."
+++

# pci_devices

## Description

Lists PCI devices from the system's `lspci` output. Reports each device's bus address, device class, vendor name, device name, and driver in use. Useful for hardware inventory, driver compatibility checks, and troubleshooting device recognition issues.

## Parameters

This tool accepts no parameters.

## Returns

### Success

```text
PCI devices listed

total: 12

device:
  slot: "00:00.0"
  class: "Host bridge"
  vendor: "Advanced Micro Devices, Inc. [AMD]"
  device: "Starship/Matisse Root Complex"
  driver: ""

device:
  slot: "00:01.0"
  class: "PCI bridge"
  vendor: "Advanced Micro Devices, Inc. [AMD]"
  device: "Starship/Matisse PCIe GPP Bridge"
  driver: "pcieport"

device:
  slot: "01:00.0"
  class: "VGA compatible controller"
  vendor: "NVIDIA Corporation"
  device: "GP107 [GeForce GTX 1050 Ti]"
  driver: "nvidia"

device:
  slot: "02:00.0"
  class: "Network controller"
  vendor: "Intel Corporation"
  device: "Wi-Fi 6 AX200"
  driver: "iwlwifi"

device:
  slot: "03:00.0"
  class: "Ethernet controller"
  vendor: "Realtek Semiconductor Co., Ltd."
  device: "RTL8111/8168/8411 PCI Express Gigabit Ethernet Controller"
  driver: "r8169"
```

### Failure

```text
PCI devices listing failed

Error: Command not found
Message: lspci is not available. Install pciutils to enable PCI device listing.
```

## Examples

### Example 1: List all PCI devices

```text
```

## Important Notes

- **Linux-only**: This tool depends on `lspci` from the `pciutils` package, which is specific to Linux
- **Dependency**: The `pciutils` package must be installed. On most distributions it can be installed via `apt install pciutils`, `yum install pciutils`, or equivalent
- **Driver field**: An empty driver field indicates the device is recognized but no kernel driver is currently bound to it, which may indicate a missing or unloaded driver
- **Permissions**: Basic `lspci` output does not require root privileges. Verbose modes may require elevated access
- **PCI database**: Device and vendor names depend on the `pci.ids` database. An outdated database may show unknown device names
