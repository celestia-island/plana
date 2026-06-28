+++
title = "Provider TOML Configuration System Design"
description = """The Provider TOML Configuration System migrates all LLM Provider configuration from hardcoded values to TOML configuration files, achieving separation of configuration and code, improving maintainabil"""
lang = "en"
category = "design"
subcategory = "core"
+++

# Provider TOML Configuration System Design

## Overview

The Provider TOML Configuration System migrates all LLM Provider configuration from hardcoded values to TOML configuration files, achieving separation of configuration and code, improving maintainability and extensibility.

## Core Objectives

| Objective | Description |
| --- | --- |
| Maintainability | Configuration separated from code, no recompilation needed for changes |
| Extensibility | Adding new Provider only requires adding TOML file |
| Readability | Configuration files are clear and easy to understand |
| Reusability | Configuration can be shared across different environments |

## Architecture Design

### Configuration Loading Process

```mermaid
flowchart TB
    subgraph Initialization Phase
        A[Application Start] --> B[Scan res/ Directory]
        B --> C[Load All .toml Files]
        C --> D[Parse TOML Structure]
    end

    subgraph Validation Phase
        D --> E{Validate Configuration Completeness}
        E -->|Pass| F[Store in Config Cache]
        E -->|Fail| G[Log Error]
        G --> H[Use Default Config]
    end

    subgraph Runtime
        F --> I[Provider Request]
        I --> J[Get Config from Cache]
        J --> K[Return ProviderConfig]
    end
```

### Configuration Hierarchy

```mermaid
graph TB
    subgraph ProviderConfig
        A[Provider Info]
        B[API Config]
        C[Limits Config]
        D[Pricing Config]
        E[Capabilities Config]
        F[Model List]
    end

    A --> A1[id, name, type, protocol]
    B --> B1[base_url, endpoints, auth]
    C --> C1[concurrency limits, rate limits, timeout]
    D --> D1[billing mode, quota info]
    E --> E1[streaming, vision, function_calling]
    F --> F1[ModelConfig List]

    subgraph ModelConfig
        F1 --> M1[id, name, context_window]
        F1 --> M2[capability support flags]
        F1 --> M3[pricing info]
        F1 --> M4[benchmark data]
    end
```

## Configuration Priority

```mermaid
graph LR
    A[User Config] -->|Highest Priority| D[Effective Config]
    B[Community Config] -->|Medium Priority| D
    C[Official Config] -->|Base Priority| D

    style A fill:#90EE90
    style B fill:#FFD700
    style C fill:#87CEEB
```

### Priority Merge Rules

| Layer | Source | Description |
| --- | --- | --- |
| 1 | Official Config | Provider official documentation data, as base defaults |
| 2 | Community Config | Community contributed optimized config, overrides official data |
| 3 | User Config | User-defined config, highest priority |

## Pricing Models

```mermaid
stateDiagram-v2
    [*] --> PayAsYouGo: Pay Per Use
    [*] --> OneTime: One-time Purchase
    [*] --> Periodic: Periodic Quota
    [*] --> Free: Free

    PayAsYouGo --> Meter Usage
    OneTime --> Check Balance
    Periodic --> Check Period Quota
    Free --> Unlimited
```

### Pricing Model Comparison

| Model | Applicable Scenarios | Characteristics |
| --- | --- | --- |
| PayAsYouGo | OpenAI, Anthropic | Pay per token, real-time deduction |
| OneTime | Prepaid packages | Pre-purchase quota, use until exhausted |
| Periodic | GLM China, etc. | Periodic quota reset |
| Free | Ollama local models | No cost limits |

## Provider Type Classification

```mermaid
graph TB
    subgraph Cloud Providers
        A[OpenAI Compatible Protocol]
        B[Anthropic Protocol]
        C[Google Gemini Protocol]
    end

    subgraph Local Providers
        D[Ollama]
        E[LocalAI]
    end

    subgraph Custom Providers
        F[User-defined Endpoints]
    end

    A --> A1[OpenAI, DeepSeek, Qwen]
    B --> B1[Claude Series]
    C --> C1[Gemini Series]
```

## Hot Reload Mechanism

```mermaid
sequenceDiagram
    participant FS as File System
    participant Watcher as Config Watcher
    participant Cache as Config Cache
    participant App as Application

    FS->>Watcher: File Change Event
    Watcher->>Watcher: Parse Changed Content
    Watcher->>Cache: Update Cache
    Cache->>App: Send Config Update Notification
    App->>App: Apply New Config
```

## Error Handling Strategy

```mermaid
flowchart TB
    A[Config Loading] --> B{Parse Success?}
    B -->|Yes| C[Validate Config]
    B -->|No| D[Log Parse Error]

    C --> E{Validation Pass?}
    E -->|Yes| F[Store in Cache]
    E -->|No| G[Log Validation Error]

    D --> H[Use Default Config]
    G --> H

    F --> I[Normal Use]
    H --> I
```

## Extensibility Design

### Adding New Provider

```mermaid
flowchart LR
    A[Create TOML File] --> B[Define Provider Info]
    B --> C[Configure API Endpoints]
    C --> D[Add Model List]
    D --> E[Set Pricing Info]
    E --> F[Restart Application]
    F --> G[Auto Load Config]
```

### Configuration Validation Rules

| Field | Validation Rule | Error Handling |
| --- | --- | --- |
| provider.id | Non-empty, unique | Reject loading, log error |
| api.base_url | Valid URL format | Use default value |
| models[].id | Non-empty | Skip that model |
| pricing.model | Enum value check | Default PayAsYouGo |

## Security Considerations

```mermaid
flowchart TB
    subgraph Sensitive Info Handling
        A[API Key] --> B[Encrypted Storage]
        B --> C[Use in Memory]
        C --> D[Log Masking]
    end

    subgraph Access Control
        E[Config Read] --> F{Permission Check}
        F -->|Has Permission| G[Return Config]
        F -->|No Permission| H[Deny Access]
    end
```

## Future Extensions

| Feature | Description | Priority |
| --- | --- | --- |
| Config Hot Reload | Load external config files at runtime | High |
| Config Validation | Validate config completeness at startup | High |
| Config Merging | User config overrides default config | Medium |
| Config Import/Export | Support config file import/export | Medium |
| Agent Update | Auto-update config from official docs | Low |

# Provider Metadata Management Design

## Overview

The Provider Metadata Management system is responsible for dynamically fetching configuration information from official LLM Provider documentation, enabling automated updates and validation of configuration data.

## Core Problem

The current implementation contains hardcoded usage statistics and lacks dynamic Provider data support. An automated metadata acquisition and management mechanism needs to be established.

## Architecture Design

### Data Flow Architecture

```mermaid
flowchart TB
    subgraph Data Sources
        A[Official Docs]
        B[API Endpoints]
        C[Community Contributions]
    end

    subgraph Collection Layer
        D[Config Agent]
        E[Web Scraper]
        F[API Client]
    end

    subgraph Processing Layer
        G[Data Parser]
        H[Validation Engine]
        I[Merge Strategy]
    end

    subgraph Storage Layer
        J[Config Database]
        K[Cache Layer]
    end

    A --> D
    B --> F
    C --> D
    D --> G
    E --> G
    F --> G
    G --> H
    H --> I
    I --> J
    J --> K
```

### Configuration Priority Model

```mermaid
graph TB
    subgraph Priority Layers
        A[User Config] -->|Highest| D[Effective Config]
        B[Community Config] -->|Medium| D
        C[Official Config] -->|Base| D
    end

    subgraph Merge Rules
        D --> E[Field-level Override]
        E --> F[Keep High Priority Value]
    end
```

## Metadata Structure

### Provider Configuration Hierarchy

```mermaid
classDiagram
    class ProviderConfig {
        +provider_id: String
        +display_name: String
        +available_models: List~ModelConfig~
        +default_model: String
        +pricing_model: PricingModel
        +usage_type: UsageType
        +api_endpoint: String
    }

    class ModelConfig {
        +model_id: String
        +model_name: String
        +context_window: u64
        +max_output_tokens: u64
        +supports_vision: bool
        +supports_function_calling: bool
    }

    class PricingModel {
        <<enumeration>>
        OneTime
        Periodic
        PayAsYouGo
    }

    class UsageType {
        <<enumeration>>
        Metered
        Quota
        Unlimited
    }

    ProviderConfig --> ModelConfig
    ProviderConfig --> PricingModel
    ProviderConfig --> UsageType
```

### Configuration Source Classification

| Source Type | Description | Reliability | Update Frequency |
| --- | --- | --- | --- |
| Official | Provider official documentation | High | Automatic periodic |
| Community | Community contributed data | Medium | Manual update |
| UserOverride | User customized | Highest | Real-time |

## Agent Collection System

### Collection Process

```mermaid
sequenceDiagram
    participant Scheduler as Scheduler
    participant Agent as Config Agent
    participant Source as Data Source
    participant Parser as Parser
    participant Validator as Validator
    participant DB as Database

    Scheduler->>Agent: Trigger collection task
    Agent->>Source: Request official docs
    Source-->>Agent: Return HTML/JSON
    Agent->>Parser: Parse content
    Parser-->>Agent: Structured data
    Agent->>Validator: Validate data
    Validator-->>Agent: Validation result
    Agent->>DB: Store config
    DB-->>Agent: Store success
    Agent-->>Scheduler: Task complete
```

### Provider Agent Responsibilities

```mermaid
flowchart LR
    subgraph OpenAI Agent
        A1[Get model list]
        A2[Parse pricing info]
        A3[Extract rate limits]
    end

    subgraph Anthropic Agent
        B1[Get Claude models]
        B2[Parse context window]
        B3[Extract capability info]
    end

    subgraph GLM Agent
        C1[Get GLM models]
        C2[Parse quota info]
        C3[Extract reset period]
    end
```

## Data Validation Mechanism

### Validation Process

```mermaid
flowchart TB
    A[Receive config data] --> B{Format validation}
    B -->|Pass| C{Logic validation}
    B -->|Fail| D[Log error]

    C -->|Pass| E{Completeness validation}
    C -->|Fail| D

    E -->|Pass| F{Consistency validation}
    E -->|Fail| G[Fill defaults]

    F -->|Pass| H[Accept config]
    F -->|Fail| I[Mark for review]

    G --> F
    D --> J[Reject config]
```

### Validation Rules

| Validation Type | Check Content | Failure Handling |
| --- | --- | --- |
| Format validation | Data types, field formats | Reject and log |
| Logic validation | Value ranges, enum values | Use default values |
| Completeness validation | Required fields exist | Fill default values |
| Consistency validation | Cross-field relationships correct | Mark for review |

## Configuration Merge Strategy

### Field-level Merge

```mermaid
flowchart TB
    subgraph Input
        A[Official Config]
        B[Community Config]
        C[User Config]
    end

    subgraph Merge Process
        D[By field priority]
        E[Keep non-null values]
        F[Validate result]
    end

    A --> D
    B --> D
    C --> D
    D --> E
    E --> F
    F --> G[Effective Config]
```

### Merge Example

| Field | Official Value | Community Value | User Value | Final Value |
| --- | --- | --- | --- | --- |
| context_window | 128000 | - | 64000 | 64000 |
| max_concurrent | 100 | 50 | - | 50 |
| pricing_model | PayAsYouGo | - | - | PayAsYouGo |

## User Configuration Interface

### Configuration File Structure

```mermaid
graph TB
    subgraph User Config File
        A[Provider display name]
        B[Usage type settings]
        C[Quota limits]
        D[Concurrency control]
        E[Context management]
        F[Model overrides]
    end

    A --> A1[Custom display name]
    B --> B1[metered/quota/unlimited]
    C --> C1[Data limit/Recovery period]
    D --> D1[Max concurrent]
    E --> E1[Theoretical limit/Practical limit]
    F --> F1[Custom model list]
```

## Scheduled Update Mechanism

```mermaid
sequenceDiagram
    participant Timer as Timer
    participant Queue as Task Queue
    participant Agent as Agent Pool
    participant DB as Database

    Timer->>Queue: Add update task
    Queue->>Agent: Assign task

    loop Each Provider
        Agent->>Agent: Get latest config
        Agent->>DB: Compare changes
        alt Has changes
            DB->>DB: Update config
            DB->>DB: Log changes
        else No changes
            DB->>DB: Update check time
        end
    end

    Agent-->>Queue: Task complete
```

## Error Handling

### Collection Failure Handling

```mermaid
flowchart TB
    A[Collection failed] --> B{Failure type}
    B -->|Network error| C[Retry mechanism]
    B -->|Parse error| D[Log and skip]
    B -->|Validation error| E[Mark for review]

    C --> F{Retry count}
    F -->|Not exceeded| G[Delayed retry]
    F -->|Exceeded| H[Use cached data]

    G --> A
    D --> I[Continue next]
    E --> J[Manual review queue]
```

## Extensibility Design

### Adding New Provider

```mermaid
flowchart LR
    A[Define Agent] --> B[Implement collection interface]
    B --> C[Configure parse rules]
    C --> D[Register to scheduler]
    D --> E[Start collection]
```

### Extension Points

| Extension Type | Description | Implementation |
| --- | --- | --- |
| New Provider | Add new config source | Implement Provider Agent interface |
| New field | Extend config structure | Update data model and validation rules |
| New validation rule | Add validation logic | Add validator implementation |

## Layer3 Agent Implementation

### ProviderScratch Agent

`ProviderScratch` is the first Layer3 official Agent, serving as an example implementation of scraping facilities.

```mermaid
flowchart TB
    subgraph ProviderScratch Agent
        A[Agent Entry] --> B{Execution Mode}
        B -->|TUI Mode| C[Interactive Interface]
        B -->|CI Mode| D[Automated Execution]

        C --> E[Select Provider]
        D --> F[Read env vars]

        E --> G[Call Skill]
        F --> G

        G --> H[Scrape docs]
        H --> I[Parse data]
        I --> J[Generate TOML]

        J --> K{Confirm commit?}
        K -->|Yes| L[Write to workspace]
        K -->|No| M[Discard changes]

        L --> N[Request user commit]
    end
```

### Skill Architecture

Each Provider corresponds to an independent Skill:

```mermaid
graph LR
    subgraph Skills
        A[openai]
        B[anthropic]
        C[glm]
        D[deepseek]
        E[qwen]
        F[gemini]
    end

    subgraph Shared Components
        G[Doc Scraper]
        H[Data Parser]
        I[TOML Generator]
    end

    A --> G
    B --> G
    C --> G
    D --> G
    E --> G
    F --> G

    G --> H
    H --> I
```

### Directory Structure

```mermaid
flowchart LR
    Root[".amphoreus/provider_scratch/"]
    AT["agent.toml"]
    OV["overview/"]
    SK["skills/"]
    Root --> AT
    Root --> OV
    Root --> SK
    OV --> ZH["zhs.md"]
    SK --> OA["openai/"]
    SK --> AN["anthropic/"]
    SK --> GL["glm/"]
    SK --> DS["deepseek/"]
    SK --> QW["qwen/"]
    SK --> GE["gemini/"]
    OA --> OAP["prompt.md"]
    AN --> ANP["prompt.md"]
    GL --> GLP["prompt.md"]
    DS --> DSP["prompt.md"]
    QW --> QWP["prompt.md"]
    GE --> GEP["prompt.md"]
```

### CI Automation

```mermaid
flowchart LR
    A[Scheduled trigger] --> B[Checkout code]
    B --> C[Run ProviderScratch]
    C --> D{Detect changes}
    D -->|Has changes| E[Create branch]
    E --> F[Commit changes]
    F --> G[Create PR]
    G --> H[Wait for review]
    D -->|No changes| I[Complete]
```

### Environment Variables

| Variable Name | Description |
| --- | --- |
| `AMPHOREUS_PROVIDER_SCRATCH_PROVIDERS` | List of providers to scrape |
| `AMPHOREUS_PROVIDER_SCRATCH_OUTPUT_DIR` | Output directory path |
| `AMPHOREUS_PROVIDER_SCRATCH_GIT_BRANCH` | Target Git branch |
| `AMPHOREUS_PROVIDER_SCRATCH_DRY_RUN` | Dry run only |

## Future Plans

| Feature | Description | Priority |
| --- | --- | --- |
| Config version control | Track config change history | High |
| Change notification | Notify users on config updates | Medium |
| Config rollback | Support rollback to historical versions | Medium |
| Smart recommendations | Recommend configs based on usage patterns | Low |
| GitHub巡回 Agent | Auto-create PRs to update configs | High |
