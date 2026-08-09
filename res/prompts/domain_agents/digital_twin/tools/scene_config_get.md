+++
name = "scene_config_get"
agent = "digital_twin"

[description]
en = "Read the twin scene configuration (background, ground, lighting, camera)"
zh-Hans = "读取孪生场景配置（背景/地面/光照/相机）"
+++

# scene_config_get

## Description

Reads `projects.sceneConfig.get` for the current project — background
color, ground size/color/grid, lighting intensities and camera
position/target/fov. Returns the full scene config JSON.

## Example

```json
{ "projectId": "00000000-0000-0000-0000-000000000001" }
```
