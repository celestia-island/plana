+++
name = "serial_discover"
agent = "industrial_iot"

[description]
en = "Enumerate serial ports and discover Modbus RTU devices (baud sweep + station scan)."
+++

# serial_discover

## Description

Two combinable modes:

1. **Port enumeration** (default): lists serial ports via evernight's

`enumerate_ports()`, returning port metadata (vid/pid/manufacturer/product).

1. **Modbus RTU probing** (`probe_baud=true` / `scan_stations=true`): sweeps

standard baud rates for a responsive Modbus RTU slave, and optionally scans
a configurable station-id range.

This tool is **read-only**: it never writes to a device.

## Parameters

- **port** (string, optional): filter to a single port path.
- **`probe_baud`** (bool, optional): if true, probe baud rates on each port. Default: `false`.
- **`scan_stations`** (bool, optional): if true (implies `probe_baud`), scan

Modbus station ids in `station_start`..=`station_end`. Default: `false`.

- **`station_start`** (number, optional): first station id. Default: `1`.
- **`station_end`** (number, optional): last station id. Default: `10` (a full

1–247 sweep is very slow over serial).

- **`baud_rates`** (array, optional): explicit list of baud rates to sweep.

## Returns

- `ports` (array): enumerated port metadata.
- `count` (number): number of ports.
- `devices` (array): discovered Modbus RTU devices

`{ port_name, protocol, station_id, baud }`.

- `device_count` (number).
