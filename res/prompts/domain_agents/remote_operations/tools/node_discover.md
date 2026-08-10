+++
name = "node_discover"
agent = "remote_operations"

[description]
en = "Discover and scan available nodes in the network."
+++

# node_discover

## Description

Scans the network to discover available nodes within a specified subnet or address range. Reports each discovered node's identifier, address, open ports, and connectivity status. Useful for network topology mapping and automated node onboarding.

## Parameters

- **subnet** (string, optional): Network subnet to scan in CIDR notation (e.g. `"198.51.100.0/24"`). Default: local subnet of the host
- **`port_range`** (string, optional): Range of ports to scan on each host. Format: `"start-end"` (e.g. `"22-80"`). Default: `"22"`
- **timeout** (number, optional): Maximum time in seconds to wait for a response from each host before moving on. Default: `5`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Node discovery completed

found: 3 nodes

node_id: "node-001"
address: "198.51.100.10"
open_ports: [22, 80, 443]
status: "reachable"

node_id: "node-002"
address: "198.51.100.20"
open_ports: [22]
status: "reachable"

node_id: "node-003"
address: "198.51.100.30"
open_ports: [22, 8080]
status: "reachable"
```

### Failure

```text
Node discovery failed

Error: Invalid subnet format
Message: Expected CIDR notation (e.g. "198.51.100.0/24"), got "198.51.100".
```

## Examples

### Example 1: Scan default local subnet

```text
```

### Example 2: Targeted subnet with custom port range

```text
subnet: "192.0.2.0/24"
port_range: "22-443"
timeout: 10
```

### Example 3: Quick scan for SSH-accessible nodes

```text
subnet: "172.16.0.0/16"
port_range: "22"
timeout: 3
```

## Important Notes

- **Network permissions**: The scanning host must have network-level access to the target subnet. Firewalls or network segmentation may block discovery probes
- **Scan duration**: Wide subnets (e.g. `/16`) and broad port ranges can take a long time to complete. Narrow the scope or increase the timeout for reliable results
- **Timeout tuning**: A short timeout may miss slow-to-respond nodes. Increase the timeout for networks with high latency
- **Auto-registration**: Discovered nodes are not automatically registered. Use `node_connect` to establish a persistent connection to nodes of interest
