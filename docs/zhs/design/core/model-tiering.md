+++
title = "模型分级系统设计"
description = "模型分级系统是一个智能模型选择机制，根据任务复杂度匹配适当的模型等级，在保证质量的同时最大化资源利用效率。"
lang = "zhs"
category = "design"
subcategory = "core"
+++

# 模型分级系统设计

## 概述

模型分级系统是一个智能模型选择机制，根据任务复杂度匹配适当的模型等级，在保证质量的同时最大化资源利用效率。

> **相关文档**：本文定义的三级模型体系是[自我进化循环系统](04-self-evolution-loop.md)的基础。

## 核心原则

### 三级模型体系

```mermaid
graph TB
    subgraph 模型等级
        T1[T1 深度思考<br/>复杂推理]
        T2[T2 常规思考<br/>标准任务]
        T3[T3 基础思考<br/>原子操作]
    end

    T1 --> |降级| T2
    T2 --> |降级| T3
    T3 --> |微调进化| T3_Fine[微调后的 T3]
```

### 等级对比

| 等级 | 定位 | 成本 | 典型场景 |
| --- | --- | --- | --- |
| T1（深度） | 复杂推理、决策 | 最高 | 架构设计、问题分析 |
| T2（常规） | 标准任务 | 中等 | 代码编写、文档生成 |
| T3（基础） | 原子操作 | 最低 | 文件读取、格式转换 |

## 模型选择机制

### 选择流程

```mermaid
flowchart TD
    Request[任务请求] --> Parse[解析 level 字段]
    Parse --> Filter{筛选匹配等级的模型}
    Filter --> |有可用模型| Check{检查配额}
    Filter --> |无可用模型| Downgrade[尝试降级]
    Check --> |配额充足| Select[选择最高优先级]
    Check --> |配额耗尽| Next[尝试下一个]
    Select --> Execute[执行任务]
    Downgrade --> Filter
    Next --> Check
```

### 降级策略

```mermaid
stateDiagram-v2
    [*] --> Deep: deep 任务
    Deep --> Normal: deep 模型不可用
    Normal --> Basic: normal 模型不可用
    Basic --> [*]: 执行或报错

    Deep --> [*]: 执行成功
    Normal --> [*]: 执行成功
```

## 配置机制

### Skill/MCP 等级标注

每个 Skill 和 MCP 工具通过 `level` 字段声明所需的模型等级：

```mermaid
graph LR
    subgraph 配置层
        S[Skill 配置]
        M[MCP 配置]
    end

    subgraph level 字段
        L[level: deep/normal/basic]
    end

    S --> L
    M --> L
    L --> |运行时| Select[模型选择器]
```

### 优先级控制

```mermaid
graph LR
    subgraph 优先级因素
        A[用户配置优先级]
        B[模型等级匹配]
        C[配额状态]
    end

    A --> |最高权重| Sort[排序]
    B --> |次要权重| Sort
    C --> |过滤条件| Filter[过滤]
    Filter --> Sort
    Sort --> Select[选择模型]
```

## 与其他模块的关系

```mermaid
graph TB
    A[模型分级系统] --> B[自我进化循环]
    A --> C[周期用量统计]
    A --> D[成本报告]

    B --> |微调目标| A
    C --> |选择依据| A
    D --> |成本数据| A
```

## 设计考量

### 成本优化

- 优先选择低等级模型
- 自动降级避免任务失败
- 配额监控告警

### 质量保证

- 复杂任务要求高等级
- 降级需进行可行性验证
- 失败时自动重试

### 可扩展性

- 支持自定义等级
- 灵活的优先级配置
- 可插拔的选择策略
