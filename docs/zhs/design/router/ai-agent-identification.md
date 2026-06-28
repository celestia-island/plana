# AI 智能体标识与提交共同作者策略

## 概述

`evernight` 以两种方式参与 celestia-island 的共同作者策略：

1. **作为提交宿主**：当 AI 智能体通过 evernight 编排一次提交（主机 A 上的智能体 →
   evernight SSH/exec → 主机 B → `git commit`）时，主机侧的 `commit-msg` 钩子（由
   `noa` 安装）会在本地触发并为提交打上溯源元数据。
2. **作为中转提供商**：当 evernight 中转模型流量时，它可以作为服务方平台出现在作者邮箱
   中，使传输跳可被审计。

本文档规定 evernight 的角色。权威机制定义于 `noa` 的设计文档；此处涵盖 evernight 专属的
集成。

## 提供商标识模型

作者邮箱使用 `celestia.world` 信任命名空间：

```
显示名 <provider-or-platform-id@celestia.world>
```

当 evernight 中转一个模型时，提供商 id 反映该中转：

```
GLM 5 <evernight.celestia.world@celestia.world>   # 经 evernight 中转的 GLM 5
```

第一方提供商保留自己的域名（`anthropic.com`、`deepseek.com`、`zhipuai.cn`……）；第三方中转
保留自己的（`opencode.ai`、`jdcloud.com`、`openrouter.ai`……）。这使"哪个模型、经由谁"
的链条在每次提交上可见。

## 共同作者 Trailer

- Trailer 键：`Co-authored-by`（git 识别）。
- 每个不同模型一行，按使用顺序排列。
- 完全在 YOLO 巡航控制下运行的链路额外获得：
  `Co-authored-by: Entelecheia <demiurge@celestia.world>`。

## 内嵌 Token 用量

追加在共同作者 trailer 之后（空行分隔）：

```
Co-authored-by: Claude Opus 4.8 (↑ 12.5k ↓ 8.3k ●45.2k) <anthropic.com@celestia.world>
Co-authored-by: Deepseek V4 Pro (↑ 5.1k ↓ 3.2k) <deepseek.com@celestia.world>
```

- `Upload` = 输入 token；`Download` = 输出 token。
- `Cache` 仅在缓存输入 token 被上报且 > 0 时才出现。
- 计数以千为单位（`k`），保留一位小数，去除尾部零。

## evernight 集成点

### 主机侧钩子

通过 `evernight` 的 `Command.Exec` JSON-RPC（被 entelecheia 的手术管线和 `KaLos:auto_fix`
循环使用）产出的提交调用系统 `git`，因此由 `noa hook install` 安装的
`.git/hooks/commit-msg` 钩子原样适用。对于在已安装钩子的主机上进行的提交，无需修改
evernight 代码。

### 中转提供商身份

当 evernight 代理 LLM 流量（例如将模型调用路由到远端主机的本地推理）时，可告知共同作者
解析器该中转端点，使提供商 id 成为 `evernight.celestia.world`。这通过 `noa co-author
resolve` 读取的同一份 `aporia.toml` 提供商列表配置。

## 完整提交消息示例

```
perf(screen): cache X11 connection to avoid per-frame reconnect

X11CaptureBackend previously called x11rb::connect on every capture_frame.
Cache the connection in a Mutex<Option<..>>, reusing it across frames.

Co-authored-by: Entelecheia <demiurge@celestia.world>
Co-authored-by: Deepseek V4 Pro (↑ 18.2k ↓ 2.1k) <deepseek.com@celestia.world>
```

## 安全考量

- 共同作者 trailer 是自报告溯源，非密码学证明。
- 解析器安全降级：缺失 `noa` 或解析错误产出空块，提交不受影响。
- 提供商标识来自本地 `aporia.toml`，反映已配置的提供商。

## 提供商标识参考（初始注册表）

| 提供商 id | 品牌 | 端点提示 |
| --- | --- | --- |
| `zhipuai.cn` | GLM | `open.bigmodel.cn` |
| `deepseek.com` | Deepseek | `api.deepseek.com` |
| `anthropic.com` | Claude | `api.anthropic.com` |
| `openai.com` | GPT / OpenAI | `api.openai.com` |
| `evernight.celestia.world` | （中转） | evernight 代理 |
| `opencode.ai` | （中转） | `opencode.ai` |
| `jdcloud.com` | （中转） | `jdcloud.com` |
