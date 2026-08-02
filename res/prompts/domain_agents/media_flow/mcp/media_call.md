+++
name = "media_call"
agent = "media_flow"

[description]
en = "Call a media generation endpoint directly (chat, image, 3D, register)"
zh-Hans = "直接调用媒体生成端点（对话/图像/3D/注册）"
+++

# media_call

## Description

Invokes one of the chest `media.*` RPCs directly:

- `media.llm_chat` — vision critique with a multimodal model
- `media.gen_image` — text-to-image (CogView)
- `media.gen_3d` — text/image-to-3D (requires TRELLIS deployment)
- `media.register_model` — persist a GLB as a DeviceModel

## Parameters

- `method`: one of the four above
- `payload`: method-specific parameters

## Example

```json
{
  "method": "media.gen_image",
  "payload": { "prompt": "high-pressure hydrogen pump, PBR", "model": "cogview" }
}
```
