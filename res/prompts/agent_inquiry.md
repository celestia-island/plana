+++
name = "Agent 问询系统"
description = "该 Skill 实现了 Agent 之间的双向问询机制，支持子 Agent 向父 Agent 或同级 Agent 请求决策指导、进度查询和信息同步。"
+++

# Agent 问询系统

## 概述

该 Skill 实现了 Agent 之间的双向问询机制，支持子 Agent 向父 Agent 或同级 Agent 请求决策指导、进度查询和信息同步。

## 核心功能

- **向上问询**：子 Agent 向父 Agent 请求决策指导
- **同级问询**：同级 Agent 之间查询进度和状态
- **快照上下文**：问询时复用目标 Agent 完成时的快照上下文
- **异步回复**：支持异步问询和回复机制
- **问询追踪**：记录所有问询和回复的历史

## 使用场景

- Agent 在关键决策点需要人类或上级确认
- 子任务需要了解父任务的目标和约束
- 同级任务之间需要协调和同步
- 查询其他 Agent 的执行进度和结果

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `timeout` | int | 300 | 问询超时时间（秒） |
| `max_retries` | int | 3 | 最大重试次数 |
| `priority` | string | "normal" | 问询优先级：low/normal/high/urgent |
| `use_snapshot` | bool | true | 是否使用快照上下文 |

## 示例

```python
# 向父 Agent 问询
inquiry = agent_inquiry(
    target_agent="parent",
    question="是否应该优先处理性能优化还是功能完整性？",
    context={
        "current_progress": 0.6,
        "options": ["性能优化", "功能完整性"],
        "constraints": ["时间限制：2周"]
    }
)

# 同级 Agent 进度查询
progress = agent_inquiry(
    target_agent="data_collector",
    question="当前数据收集进度如何？",
    inquiry_type="progress_check"
)
```

## 最佳实践

1. 明确问询的目的和期望的回复类型
1. 提供充分的上下文信息
1. 设置合理的超时时间
1. 对于紧急问询，设置高优先级
1. 定期检查未回复的问询
1. 使用快照功能减少重复解释

## 问询类型

- **`decision_guidance`**：决策指导
- **`progress_check`**：进度查询
- **`resource_request`**：资源请求
- **`conflict_resolution`**：冲突解决
- **`information_sync`**：信息同步
