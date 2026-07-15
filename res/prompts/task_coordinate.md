+++
name = "任务编排与容器生命周期管理"
description = "该 Skill 负责协调多个 Agent 之间的任务分配，并管理容器的完整生命周期，包括创建、启动、停止和销毁。"
+++

# 任务编排与容器生命周期管理

## 概述

该 Skill 负责协调多个 Agent 之间的任务分配，并管理容器的完整生命周期，包括创建、启动、停止和销毁。

## 核心功能

- **多 Agent 编排**：智能分配任务给不同的 Agent，实现并行处理和负载均衡
- **容器生命周期管理**：完整的容器生命周期控制
- 容器创建与初始化
- 启动与运行监控
- 停止与重启
- 资源清理与销毁
- **任务调度**：基于优先级和依赖关系的智能任务调度
- **状态同步**：实时同步各 Agent 和容器的状态信息

## 使用场景

- 复杂的多步骤任务需要多个 Agent 协同完成
- 需要动态创建和管理容器环境
- 分布式任务处理与编排

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `max_agents` | int | 10 | 最大并发 Agent 数量 |
| `timeout` | int | 3600 | 任务超时时间（秒） |
| `retry_policy` | string | "exponential" | 重试策略 |

## 示例

```python
# 编排多个 Agent 执行任务
result = task_coordinate(
    agents=["data_collector", "analyzer", "reporter"],
    task="生成月度报告",
    dependencies={"reporter": ["analyzer"], "analyzer": ["data_collector"]}
)
```

## 最佳实践

1. 明确定义 Agent 之间的依赖关系
1. 设置合理的超时和重试策略
1. 监控容器资源使用情况
1. 定期清理不再使用的容器
