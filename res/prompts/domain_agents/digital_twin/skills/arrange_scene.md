+++
name = "arrange_scene"
agent = "digital_twin"
execution_mode = "read"

[description]
en = "Lay out or reposition models in the 3D twin scene"
zh-Hans = "在 3D 孪生场景中布置或重排模型"
+++

# Arrange Scene

Plan and apply model placement for the holographic twin: inspect the
current model list (`projects.deviceModels.list`), decide world
coordinates for new or moved models (avoiding overlaps, keeping the
box grouping convention), then apply via `model_place`.

Use `scene_config_get` to check camera/grid bounds before placing.
