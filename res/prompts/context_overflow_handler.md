+++
name = "上下文溢出处理"
description = "该 Skill 在对话上下文接近或超过限制时，智能压缩和摘要历史内容，确保关键信息不丢失。"
+++

# 上下文溢出处理

## 概述

该 Skill 在对话上下文接近或超过限制时，智能压缩和摘要历史内容，确保关键信息不丢失。

## 核心功能

- **智能检测**：实时监控上下文使用情况，预测溢出风险
- **分层压缩**：
- 保留关键对话节点
- 压缩重复和次要信息
- 合并相似内容
- **摘要生成**：为被压缩内容生成简洁摘要
- **优先级管理**：根据重要性决定内容保留策略
- **无缝切换**：在压缩过程中保持对话连贯性

## 使用场景

- 长时间多轮对话
- 复杂任务的上下文管理
- 大量历史记录需要保留
- 防止 token 限制导致的任务中断

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `threshold` | float | 0.85 | 触发压缩的阈值（占最大上下文的比例） |
| `compression_ratio` | float | 0.3 | 目标压缩比例 |
| `preserve_recent` | int | 10 | 保留最近 N 轮对话 |
| `strategy` | string | "hierarchical" | 压缩策略：hierarchical/semantic/chunk |

## 示例

```python
# 上下文溢出处理
context = {
    "total_tokens": 7500,
    "max_tokens": 8192,
    "messages": [...],  # 大量历史消息
}

handled = context_overflow_handler(
    context=context,
    preserve_keywords=["错误", "配置", "重要决策"]
)

# 输出处理后的上下文
{
    "total_tokens": 5200,
    "compression_ratio": 0.31,
    "preserved_messages": 10,
    "summary": "之前讨论了数据库配置优化...",
    "key_points": ["决定使用 MySQL 8.0", "配置连接池大小为 100"]
}
```

## 最佳实践

1. 设置合理的触发阈值，避免过早或过晚压缩
1. 明确标记关键信息以便优先保留
1. 定期检查压缩摘要的准确性
1. 根据任务类型选择合适的压缩策略
1. 保留足够的最近对话以保证上下文连贯
