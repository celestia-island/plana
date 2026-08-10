+++
name = "protocol_probe"
agent = "industrial_iot"

[description]
en = "Probe a remote node for supported network protocols and services."
+++

# protocol_probe

## Description

Probes a remote node for supported network protocols and services by attempting connections on specified or common ports. Reports which protocols (e.g. SSH, HTTP, HTTPS, FTP, DNS) are available, along with version banners where detected. Useful for service inventory and compatibility assessment before node onboarding.

## Parameters

- **host** (string, required): Hostname or IP address of the remote node to probe (e.g. `"198.51.100.10"` or `"node-001.local"`).
- **port** (number, optional): Specific port number to probe. When set to `0` (default), the tool probes a predefined set of common service ports (22, 80, 443, 21, 25, 53, 3306, 5432, 8080, 8443). Default: `0`

> **Parameter Format**:
> All parameters are passed as a JSON object via native function calling (`tool_calls`). See the tool definition for the JSON Schema.

## Returns

### Success

```text
Protocol probe completed

host: "198.51.100.10"

port: 22
protocol: "SSH"
banner: "OpenSSH_8.9p1 Ubuntu-3ubuntu0.6"
status: "open"

port: 80
protocol: "HTTP"
banner: "nginx/1.24.0"
status: "open"

port: 443
protocol: "HTTPS"
banner: "nginx/1.24.0"
status: "open"

port: 21
protocol: "FTP"
status: "closed"
```

### Failure

```text
Protocol probe failed

Error: Host unreachable
Message: Unable to connect to "198.51.100.999". Verify the host address and network connectivity.
```

## Examples

### Example 1: Probe all common ports

```text
host: "198.51.100.10"
```

### Example 2: Probe a specific port

```text
host: "node-001.local"
port: 22
```

### Example 3: Probe a web server

```text
host: "192.0.2.50"
port: 443
```

## Important Notes

- **Network access**: The probing host must be able to reach the target host at the network level. Firewalls, ACLs, or network segmentation may prevent probes from returning accurate results
- **Banner accuracy**: Service banners are provided on a best-effort basis. Some services may suppress or falsify banner information for security reasons
- **Port 0 behavior**: When `port` is set to `0`, the tool scans a fixed set of well-known ports. For a comprehensive port scan, use `node_discover` instead
- **Timeout**: Each individual port probe has a default timeout of 5 seconds. Hosts with high latency may report false negatives
