# Model Tiering System Design

## Overview

The Model Tiering System is an intelligent model selection mechanism that assigns appropriate model tiers based on task complexity, maximizing resource utilization while ensuring quality.

> **Related Document**: The three-tier model system defined in this document is the foundation of the [Self-Evolution Loop System](04-self-evolution-loop.md).

## Core Principles

### Three-tier Model System

```mermaid
graph TB
    subgraph Model Tiers
        T1[T1 Deep Thinking<br/>Complex Reasoning]
        T2[T2 Normal Thinking<br/>Standard Tasks]
        T3[T3 Basic Thinking<br/>Atomic Operations]
    end

    T1 --> |Degradation| T2
    T2 --> |Degradation| T3
    T3 --> |Fine-tuning Evolution| T3_Fine[Fine-tuned T3]
```

### Tier Comparison

| Tier | Positioning | Cost | Typical Scenarios |
| --- | --- | --- | --- |
| T1 (deep) | Complex reasoning, decisions | Highest | Architecture design, problem analysis |
| T2 (normal) | Standard tasks | Medium | Code writing, document generation |
| T3 (basic) | Atomic operations | Lowest | File reading, format conversion |

## Model Selection Mechanism

### Selection Process

```mermaid
flowchart TD
    Request[Task Request] --> Parse[Parse level Field]
    Parse --> Filter{Filter Matching Tier Models}
    Filter --> |Available Models| Check{Check Quota}
    Filter --> |No Available Models| Downgrade[Try Degradation]
    Check --> |Quota Sufficient| Select[Select Highest Priority]
    Check --> |Quota Exhausted| Next[Try Next One]
    Select --> Execute[Execute Task]
    Downgrade --> Filter
    Next --> Check
```

### Degradation Strategy

```mermaid
stateDiagram-v2
    [*] --> Deep: deep Task
    Deep --> Normal: deep Model Unavailable
    Normal --> Basic: normal Model Unavailable
    Basic --> [*]: Execute or Error

    Deep --> [*]: Successful Execution
    Normal --> [*]: Successful Execution
```

## Configuration Mechanism

### Skill/MCP Tier Annotation

Each Skill and MCP tool declares the required model tier through the `level` field:

```mermaid
graph LR
    subgraph Configuration Layer
        S[Skill Config]
        M[MCP Config]
    end

    subgraph Level Field
        L[level: deep/normal/basic]
    end

    S --> L
    M --> L
    L --> |Runtime| Select[Model Selector]
```

### Priority Control

```mermaid
graph LR
    subgraph Priority Factors
        A[User Config Priority]
        B[Model Tier Match]
        C[Quota Status]
    end

    A --> |Highest Weight| Sort[Sort]
    B --> |Secondary Weight| Sort
    C --> |Filter Condition| Filter[Filter]
    Filter --> Sort
    Sort --> Select[Select Model]
```

## Relationship with Other Modules

```mermaid
graph TB
    A[Model Tiering System] --> B[Self-Evolution Loop]
    A --> C[Period Usage Statistics]
    A --> D[Cost Reports]

    B --> |Fine-tuning Target| A
    C --> |Selection Basis| A
    D --> |Cost Data| A
```

## Design Considerations

### Cost Optimization

- Prioritize lower-tier models
- Automatic degradation avoids task failure
- Quota monitoring alerts

### Quality Assurance

- Complex tasks require high tier
- Degradation requires feasibility validation
- Automatic retry on failure

### Extensibility

- Support custom tiers
- Flexible priority configuration
- Pluggable selection strategies
