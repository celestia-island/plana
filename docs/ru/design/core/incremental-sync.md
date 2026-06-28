+++
title = "Архитектура Инкрементальной Синхронизации"
description = """Механизм инкрементальной синхронизации состояния нескольких клиентов на основе Automerge CRDT, поддерживающий инкрементальные обновления в реальном времени и полную синхронизацию при подключении"""
lang = "ru"
category = "design"
subcategory = "core"
+++

# Архитектура Инкрементальной Синхронизации

## Обзор

Механизм инкрементальной синхронизации состояния нескольких клиентов на основе Automerge CRDT, поддерживающий инкрементальные обновления в реальном времени и полную синхронизацию при подключении/переподключении, охватывающий все панели TUI.

## Диаграмма Архитектуры

```mermaid
flowchart TB
    subgraph Clients["Клиенты TUI (Несколько)"]
        C1["Клиент 1"]
        C2["Клиент 2"]
        C3["Клиент N"]
    end

    subgraph Server["Сервер"]
        SM["SyncManager<br/>Единое Дерево Состояний"]
        BH["BroadcastHelper"]
        WS["WebSocket Трансляция"]
        REG["StateRegistry<br/>Полное Состояние"]
    end

    subgraph Storage["Automerge CRDT"]
        AD["AgentDoc<br/>на агента"]
        V["Векторы Версий"]
    end

    %% Запросы полной синхронизации (режим pull)
    C1 -->|"При Подключении"| WS
    C2 -->|"При Подключении"| WS
    C3 -->|"При Подключении"| WS

    WS -->|"RequestFullSnapshot"| BH
    BH -->|"list_agents"| SM
    SM -->|"AgentSnapshot"| BH
    BH -->|"broadcast"| WS
    WS -->|"AgentSnapshot"| C1
    WS -->|"AgentSnapshot"| C2
    WS -->|"AgentSnapshot"| C3

    %% Инкрементальные обновления (режим push)
    SM -->|"Изменение Состояния"| BH
    BH -->|"update_agent"| SM
    SM -->|"Сгенерировать AgentPatch"| BH
    BH -->|"broadcast"| WS
    WS -->|"AgentPatch"| C1
    WS -->|"AgentPatch"| C2
    WS -->|"AgentPatch"| C3

    %% Хранилище Automerge
    SM <-->|"agent_docs"| AD
    SM <-->|"version"| V

    style SM fill:#e1f5fe
    style BH fill:#fff3e0
    style WS fill:#f3e5f5
    style AD fill:#e8f5e9
    style REG fill:#fff9c4
```

## Матрица Стратегии Синхронизации

| Панель | Метод Синхронизации | Триггер | Частота | Типы Сообщений |
| --- | --- | --- | --- | --- |
| **Временная Шкала Агентов** | Инкрементальная + Полная | Синхронизация при Подключении + Push в Реальном Времени | При Подключении / В Реальном Времени | `AgentPatch` / `GlobalSnapshot` |
| **Контейнеры** | Инкрементальная + Полная | Синхронизация при Подключении + Push в Реальном Времени | При Подключении / В Реальном Времени | `ContainerPatch` / `GlobalSnapshot` |
| **Задачи** | Инкрементальная + Полная | Синхронизация при Подключении + Push в Реальном Времени | При Подключении / В Реальном Времени | `TaskPatch` / `GlobalSnapshot` |
| **Список Моделей** | Полная | Активный Запрос Клиента | При Открытии Панели | `ModelsSnapshot` |
| **Конфигурация Провайдеров** | Полная | Активный Запрос Клиента | При Открытии Панели | `ProvidersSnapshot` |

## Поток Сообщений

### Поток Инкрементального Обновления (Агенты)

```mermaid
sequenceDiagram
    participant Agent as Среда Выполнения Агента
    participant SM as SyncManager
    participant BH as BroadcastHelper
    participant WS as WebSocket
    participant Client as Клиент TUI

    Agent->>SM: Обновление Состояния
    SM->>SM: update_agent()
    SM->>SM: Сгенерировать AgentPatch
    SM->>BH: Вернуть патч
    BH->>WS: broadcast(AgentPatch)
    WS->>Client: Сообщение AgentPatch
    Client->>Client: apply_agent_patch()
    Client->>Client: Обновить UI
```

### Поток Полной Синхронизации

```mermaid
sequenceDiagram
    participant Client as Клиент TUI
    participant WS as WebSocket
    participant BH as BroadcastHelper
    participant SM as SyncManager
    participant Registry as State Registry

    Note over Client: При Подключении / Переподключении
    Client->>WS: RequestGlobalSnapshot
    WS->>BH: send_full_snapshot()
    BH->>Registry: list_agents()
    Registry-->>BH: Vec<AgentInfo>
    BH->>SM: create_snapshot(agents)
    SM-->>BH: AgentSnapshot
    BH->>WS: broadcast(AgentSnapshot)
    WS-->>Client: Сообщение AgentSnapshot
    Client->>Client: Заменить локальное состояние
```

### Поток Синхронизации Списка Моделей

```mermaid
sequenceDiagram
    participant Client as Клиент TUI
    participant WS as WebSocket
    participant Server as Сервер

    Note over Client: При открытии панели Моделей
    Client->>WS: Запросить список Моделей
    WS->>Server: Запросить конфигурацию Моделей
    Server-->>WS: Список моделей
    WS-->>Client: Сообщение ModelsSnapshot
    Client->>Client: Обновить панель Моделей
```

### Поток Полной Синхронизации Контейнеров

```mermaid
sequenceDiagram
    participant Client as Клиент TUI
    participant WS as WebSocket
    participant Server as Сервер

    Note over Client: При открытии панели Контейнеров
    Client->>WS: RequestContainerSnapshot
    WS->>Server: Запросить состояние Контейнеров
    Server-->>WS: ContainerSnapshot
    WS-->>Client: Сообщение ContainerSnapshot
    Client->>Client: Заменить локальное состояние Контейнеров
```

### Поток Полной Синхронизации Задач

```mermaid
sequenceDiagram
    participant Client as Клиент TUI
    participant WS as WebSocket
    participant Server as Сервер

    Note over Client: При открытии панели Задач
    Client->>WS: RequestTasksSnapshot
    WS->>Server: Запросить состояние Задач
    Server-->>WS: TasksSnapshot
    WS-->>Client: Сообщение TasksSnapshot
    Client->>Client: Заменить локальное состояние Задач
```

## Структуры Данных

### AgentPatch (Инкрементальное Обновление)

```rust
pub struct AgentPatch {
    pub agent_id: String,
    pub version: u64,
    pub llm_working_changed: Option<bool>,
    pub work_status: Option<String>,
    pub current_model: Option<String>,
    pub token_usage_delta: Option<(u32, u32)>,
    pub token_usage_absolute: Option<(u32, u32)>,
    pub request_state: Option<RequestState>,
    pub cpu_usage: Option<f64>,
    pub memory_mb: Option<u64>,
}
```

### AgentSnapshot (Полный Снимок)

```rust
pub struct AgentSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
}
```

### GlobalSnapshot (Глобальный Снимок)

```rust
pub struct GlobalSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub agents: Vec<TuiAgentInfo>,
    pub models: Vec<ModelInfo>,
    pub providers: Vec<ProviderInfo>,
    pub active_tasks: Vec<TaskInfo>,
}
```

### ModelsSnapshot (Список Моделей)

```rust
pub struct ModelsSnapshot {
    pub models: Vec<ModelInfo>,
}
```

### ContainerPatch (Инкрементальное Состояние Контейнера)

```rust
pub struct ContainerPatch {
    pub container_id: String,
    pub version: u64,
    pub status_changed: Option<String>,
    pub cpu_usage_changed: Option<f64>,
    pub memory_usage_changed: Option<u64>,
}
```

### ContainerSnapshot (Полное Состояние Контейнера)

```rust
pub struct ContainerSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub containers: Vec<ContainerInfo>,
}
```

### TaskPatch (Инкрементальное Состояние Задачи)

```rust
pub struct TaskPatch {
    pub task_id: Uuid,
    pub version: u64,
    pub status_changed: Option<String>,
    pub progress_changed: Option<u8>,
}
```

### TasksSnapshot (Полное Состояние Задач)

```rust
pub struct TasksSnapshot {
    pub version: u64,
    pub timestamp: i64,
    pub tasks: Vec<TaskInfo>,
}
```

## Стратегия Синхронизации

| Тип | Направление | Триггер | Частота |
| --- | --- | --- | --- |
| Инкрементальное Обновление Агента | Сервер → Клиент | Изменение Состояния | В Реальном Времени |
| Полная Синхронизация Агента | Сервер → Клиент | При Подключении | При Подключении / Переподключении |
| Инкрементальное Обновление Контейнеров | Сервер → Клиент | Изменение Состояния | В Реальном Времени |
| Полная Синхронизация Контейнеров | Сервер → Клиент | При Подключении | При Подключении / Переподключении |
| Инкрементальное Обновление Задач | Сервер → Клиент | Изменение Состояния | В Реальном Времени |
| Полная Синхронизация Задач | Сервер → Клиент | При Подключении | При Подключении / Переподключении |
| Список Моделей | Клиент → Сервер | Активный Запрос | При открытии панели |
| Конфигурация Провайдеров | Клиент → Сервер | Активный Запрос | При открытии панели |

## Ключевые Особенности

- **Единое Дерево Состояний**: Сервер поддерживает один `SyncManager`, все клиенты получают одинаковые обновления состояния
- **Разрешение Конфликтов CRDT**: Автоматическое разрешение конфликтов на основе Automerge
- **Инкрементальные Обновления**: Передаются только изменённые поля для снижения сетевого трафика
- **Согласованность в Конечном Счёте**: Полная синхронизация при подключении гарантирует согласованность в конечном счёте
- **Pull по Требованию**: Модели и Провайдеры запрашиваются по требованию при открытии их панелей, чтобы избежать ненужной сетевой передачи
- **Синхронизация Главной Страницы**: Агенты, Контейнеры и Задачи синхронизируются при подключении, так как они видны на главной странице

## Статус Реализации

- ✅ Инкрементальная/полная синхронизация агентов
- ✅ Синхронизация списка моделей
- ✅ Синхронизация конфигурации провайдеров
- ✅ Инкрементальная/полная синхронизация контейнеров
- ✅ Инкрементальная/полная синхронизация задач
- ✅ Персистентность состояния (хранение /tmp, перезагрузка при перезапуске)
