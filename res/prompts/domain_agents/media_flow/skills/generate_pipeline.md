+++
name = "generate_pipeline"
agent = "media_flow"
execution_mode = "read"

[description]
en = "Generate a media pipeline graph from a natural-language request"
+++

# Generate Pipeline

Translate the user's goal (e.g. "generate a PBR 3D model of a pump and
critique it") into a node-graph: choose generation nodes, wire ports
type-safely (text → image → 3D → render → critique → loop → export),
then push the graph via `pipeline_run` for the user to inspect and run.
