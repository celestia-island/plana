+++
name = "节点任务总结"
description = "该 Skill 在节点任务完成后生成结构化的总结报告，记录关键信息、结果和经验教训。"
+++

# 节点任务总结

## 概述

该 Skill 在节点任务完成后生成结构化的总结报告，记录关键信息、结果和经验教训。

## 核心功能

- **结果汇总**：总结任务执行的主要成果和输出
- **性能分析**：分析任务执行的效率、资源使用和时间消耗
- **问题记录**：记录执行过程中遇到的问题和解决方案
- **经验提取**：提取可复用的经验和最佳实践
- **后续建议**：提供改进建议和后续行动项

## 使用场景

- 完成复杂任务后需要生成报告
- 项目里程碑节点的总结
- 团队协作中的信息同步
- 知识管理和经验积累

## 配置参数

| 参数 | 类型 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `detail_level` | string | "standard" | 总结详细程度：brief/standard/detailed |
| `include_metrics` | bool | true | 是否包含性能指标 |
| `include_artifacts` | bool | true | 是否列出产出物 |
| `language` | string | "zh" | 输出语言 |

## 示例

```python
# 生成任务总结
task_info = {
    "task_id": "deploy-app-v2",
    "start_time": "2024-01-15 10:00:00",
    "end_time": "2024-01-15 14:30:00",
    "status": "completed",
    "steps": [
        {"name": "代码构建", "status": "success", "duration": "15m"},
        {"name": "测试执行", "status": "success", "duration": "45m"},
        {"name": "部署上线", "status": "success", "duration": "10m"},
        {"name": "监控验证", "status": "success", "duration": "5m"}
    ]
}

summary = node_task_summary(task_info)

# 输出总结报告
{
    "overview": {
        "task": "部署应用 v2.0",
        "status": "成功完成",
        "duration": "4小时30分钟",
        "completion_rate": "100%"
    },
    "achievements": [
        "成功部署新版本应用到生产环境",
        "所有测试用例通过（156/156）",
        "零停机时间部署"
    ],
    "metrics": {
        "total_time": "4h 30m",
        "test_coverage": "92%",
        "deployment_time": "10m",
        "rollback_scenarios": 0
    },
    "issues_resolved": [
        {
            "issue": "数据库连接池配置错误",
            "solution": "调整连接池参数至推荐值",
            "impact": "防止了潜在的性能问题"
        }
    ],
    "lessons_learned": [
        "预先验证数据库配置可减少部署风险",
        "自动化测试覆盖率提升显著提高了部署信心"
    ],
    "next_steps": [
        "监控新版本性能指标",
        "收集用户反馈",
        "计划下一次迭代"
    ]
}
```

## 最佳实践

1. 及时记录，避免信息遗忘
1. 客观描述问题和解决方案
1. 量化成果和性能指标
1. 提取可复用的经验教训
1. 明确后续行动项和责任人
