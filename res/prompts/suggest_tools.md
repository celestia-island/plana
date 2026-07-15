+++
name = "工具推荐"
description = "该 Skill 根据任务需求智能推荐合适的 MCP (Model Context Protocol) 工具，提高任务执行效率。"
+++

# 工具推荐

## 概述

该 Skill 根据任务需求智能推荐合适的 MCP (Model Context Protocol) 工具，提高任务执行效率。

## 核心功能

- **任务分析**：深度分析任务需求和约束条件
- **工具匹配**：从工具库中匹配最适合的工具
- **优先级排序**：根据任务特性对推荐工具排序
- **使用建议**：提供工具使用的最佳实践建议
- **替代方案**：当首选工具不可用时推荐备选方案

## 使用场景

- 不确定应该使用哪个工具完成任务
- 需要发现新的工具来提高效率
- 优化现有工作流程
- 学习可用的工具生态

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `task_context` | dict | {} | 任务上下文信息 |
| `tool_categories` | list | [] | 限定的工具类别 |
| `max_recommendations` | int | 5 | 最大推荐数量 |
| `include_experimental` | bool | false | 是否包含实验性工具 |

## 示例

```python
# 根据任务推荐工具
task = {
    "type": "data_analysis",
    "requirements": [
        "处理大型 CSV 文件",
        "生成可视化图表",
        "导出 PDF 报告"
    ],
    "constraints": {
        "max_execution_time": 60,
        "output_format": "pdf"
    }
}

recommendations = suggest_tools(task)

# 输出推荐结果
{
    "primary": {
        "tool": "pandas_analyzer",
        "reason": "最适合大型数据处理和可视化"
    },
    "alternatives": [
        {
            "tool": "polars_processor",
            "reason": "更快的处理速度，但功能较少"
        },
        {
            "tool": "datawrapper",
            "reason": "优秀的可视化，但不支持本地处理"
        }
    ],
    "workflow": ["pandas_analyzer", "matplotlib_viz", "report_generator"]
}
```

## 最佳实践

1. 提供详细的任务描述和约束条件
1. 明确优先级和性能要求
1. 考虑工具的兼容性和依赖关系
1. 评估工具的学习曲线
1. 准备备选方案以应对工具不可用的情况
