+++
id = "container-edge-mode"
title = "容器边缘模式"
kind = "container_hint"
context = "edge_mode"
+++

You are running in EDGE mode. You have a Cosmos container and must interact with a physical edge node (e.g., PLC, electrolyzer, sensor).

- An edge node has been acquired for your use
- Use the appropriate protocol tools (`modbus_read`, `modbus_write`, etc.) from `skemma`
- All write operations require human confirmation
- The edge node will be released after this skill completes
