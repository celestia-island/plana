+++
name = "bind_telemetry"
agent = "digital_twin"
execution_mode = "read"

[description]
en = "Bind polemos telemetry stations to twin models"
+++

# Bind Telemetry

Inspect which models lack a `polemos_node_id` and bind the correct
station ids so live telemetry (temperature, pressure, power) overlays
onto the right 3D model. List stations via `polemos.node_discover`,
then apply bindings via `model_place`.
