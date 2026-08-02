+++
name = "human_requirement_parse"
description = "SkoPeo 负责将用户的人类语言需求解析为结构化任务清单，供调度层执行。"
agent = "SkoPeo"
version = "1.0.0"
+++

# 人类需求解析

## 概述

本 Skill 由 SkoPeo 提供：将用户的自由文本需求解析为结构化的任务清单，包含目标、约束、优先级与验收标准。

## 解析流程

1. 提取用户明确陈述的目标。
2. 识别隐含约束（资源、时间、权限）。
3. 将需求拆分为可执行的任务项，标注依赖关系。
4. 为每个任务项生成可验证的验收标准。

## 输出格式

- 任务清单（JSON 数组），每项包含 `id`、`goal`、`constraints`、`priority`、`acceptance_criteria`。
- 若需求存在歧义，输出 `clarification_required` 标记并列出待确认问题。

## 示例

用户输入："帮我监控产线温度，超过阈值告警"。

输出：

```json
[
  {
    "id": "task-1",
    "goal": "监控产线温度",
    "constraints": ["采样周期 10s", "数据源 Modbus FC03"],
    "priority": "high",
    "acceptance_criteria": ["温度读数可查询", "超阈值触发告警"]
  }
]
```
