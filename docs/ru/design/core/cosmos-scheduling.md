+++
title = "Проект Планирования Контейнеров Cosmos и Маршрутизации Токенов"
description = """Этот документ описывает архитектуру планирования контейнеров Cosmos: как инструменты MCP, помеченные `ToolLocation::Cosmos`, маршрутизируются через JSON-RPC Unix-сокета к соответствующим контейнер"""
lang = "ru"
category = "design"
subcategory = "core"
+++

# Проект Планирования Контейнеров Cosmos и Маршрутизации Токенов

## Обзор

Этот документ описывает архитектуру планирования контейнеров Cosmos: как инструменты MCP, помеченные `ToolLocation::Cosmos`, маршрутизируются через JSON-RPC Unix-сокета к соответствующим контейнерам, и как система токенов (номер агента) связывается с идентичностью контейнера и маршрутизацией.

## I. Модель Расположения Инструментов

### Двойная Среда Выполнения

```mermaid
flowchart LR
    subgraph Scepter["Scepter (Центральный Процесс)"]
        A1[Вызовы LLM]
        A2[Запросы RAG]
        A3[Управление Задачами]
        A4[Хранение Учётных Данных]
    end

    subgraph Cosmos["Cosmos (Контейнер на Агента)"]
        P1[Доступ к Файловой Системе]
        P2[Выполнение Скриптов]
        P3[Доступ к Оборудованию]
        P4[Сессии REPL]
    end

    Scepter -->|ToolLocation::Scepter| Local[Локальный Вызов]
    Cosmos -->|ToolLocation::Cosmos| Socket[RPC Unix-сокета]
```

### Перечисление ToolLocation

| Вариант | Место Выполнения | Транспорт |
| --- | --- | --- |
| `Scepter` (по умолчанию) | В процессе через `McpToolInvoker` | Прямой вызов функции |
| `Cosmos` | В контейнере через `CosmosConnector` | JSON-RPC Unix-сокета |

### Критерии Решения о Расположении

```mermaid
flowchart TD
    Tool[Инструмент MCP] --> Q1{Нужны Ресурсы Контейнера?}
    Q1 -->|Да: файловая система, скрипты, оборудование| Cosmos[ToolLocation::Cosmos]
    Q1 -->|Нет: LLM, RAG, управление состоянием| Scepter[ToolLocation::Scepter]
```

Инструменты, требующие ресурсов контейнера (файловая система, выполнение скриптов, доступ к оборудованию), помечаются `Cosmos`. Централизованные сервисы (LLM, RAG, управление задачами, взаимодействие с человеком) остаются `Scepter`.

## II. Система Токенов и Идентичность Контейнера

### Выделение Номера Агента

```mermaid
sequenceDiagram
    participant SM as Менеджер Цепочки Навыков
    participant AIM as AgentIdManager
    participant SC as SnowflakeContainer
    participant PC as CosmosConnector

    SM->>AIM: Запросить номер агента
    AIM-->>SM: Назначить токен 000-999
    SM->>SC: Создать контейнер для токена
    SC-->>SM: UUID контейнера + путь сокета
    SM->>PC: connect(UUID, socket_path)
    PC-->>SM: Соединение установлено
```

### Свойства Токена

| Свойство | Описание |
| --- | --- |
| Формат | Трёхзначное число: `000`-`999` |
| Выделитель | `AgentIdManager` в цепочке навыков |
| Привязка | Один токен на панель цепочки навыков |
| Отображение | Показывается в строке статистики TUI как `cosmos#NNN` |
| Персистентность | Сохраняется при перезапусках агента |

## III. Поток Маршрутизации Запросов

### Вызов MCP из TUI

```mermaid
sequenceDiagram
    participant TUI as Клиент TUI
    participant MSR as mcp_skill_router
    participant AM as AgentManager
    participant BI as BridgeInvoker
    participant BS as HapLotesBridgeServer
    participant PR as McpRouter (Cosmos)

    TUI->>MSR: McpMessage::CallTool(имя_инструмента, тип_агента, параметры)
    MSR->>AM: get_tool_location(имя_инструмента)
    AM-->>MSR: ToolLocation

    alt ToolLocation::Cosmos
        MSR->>AM: invoke_tool(имя_инструмента, тип_агента, параметры)
        AM->>BI: маршрутизировать к правильному агенту
        BI->>BS: переслать через мост HapLotes
        BS->>PR: bridge.call(имя_инструмента, параметры)
        PR-->>BS: результат
        BS-->>BI: ответ JSON-RPC
        BI-->>AM: McpToolResult
        AM-->>MSR: McpToolResult
        MSR-->>TUI: McpMessage::ToolResponse
    else ToolLocation::Scepter
        MSR->>AM: маршрутизировать через шлюз HapLotes
        AM->>AM: mcp_tools.invoke() локально
        AM-->>TUI: McpMessage::ToolResponse через WS
    end
```

### Ключевая Логика Маршрутизации

Решение о маршрутизации происходит в `mcp_skill_router.rs`:

1. Проверить `agent_manager.get_tool_location(имя_инструмента)`
1. Если `ToolLocation::Cosmos` и активен контейнеризованный режим:

   - Вызвать `agent_manager.invoke_tool()`, который маршрутизирует через `BridgeInvoker` → мост HapLotes → `McpRouter` Cosmos
   - `McpRouter` Cosmos диспетчеризует локально (skemma) или обратно в Scepter через мост для удалённых агентов
   - Вернуть `McpMessage::ToolResponse` напрямую в TUI

1. Иначе: маршрутизировать через шлюз HapLotes к процессу агента

## IV. Архитектура CosmosConnector / Моста

### Мост HapLotes (Текущий)

Мост HapLotes является **единственным каналом связи** между Scepter и контейнерами Cosmos.

```mermaid
flowchart LR
    subgraph Cosmos["Cosmos (Контейнер)"]
        MR[McpRouter] -->|ToolSource::Local| SK[skemma Boa JS]
        MR -->|ToolSource::Bridge| BC[HapLotesBridgeClient]
    end

    subgraph Scepter["Scepter (Хост)"]
        BS[HapLotesBridgeServer] --> BI[BridgeInvoker]
        BI --> AG1[Aporia]
        BI --> AG2[KaLos]
        BI --> AG3[...все агенты]
    end

    BC -->|JSON-RPC Unix-сокета| BS
```

### Пул Соединений (CosmosConnector — сторона Scepter)

```mermaid
classDiagram
    class CosmosConnector {
        -connections: RwLock~HashMap~String, CosmosConnection~~
        +connect(instance_uuid, socket_path) Result
        +invoke_tool(instance_uuid, tool_name, params) Result~Value~
        +list_tools(instance_uuid) Result~Vec~String~~
        +disconnect(instance_uuid)
    }

    class CosmosConnection {
        -transport: Mutex~JsonRpcTransport~
    }

    class JsonRpcTransport {
        +send_request(request) Result~JsonRpcResponse~
    }

    CosmosConnector --> CosmosConnection
    CosmosConnection --> JsonRpcTransport
```

### Протокол JSON-RPC

Все имена методов используют перечисление `UnixMethod` для безопасности типов на этапе компиляции:

| Вариант UnixMethod | Направление | Параметры |
| --- | --- | --- |
| `UnixMethod::McpCall` | Scepter → Cosmos | `{ tool_name, parameters }` |
| `UnixMethod::McpListTools` | Scepter → Cosmos | Нет |
| `UnixMethod::ReplSnapshot` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::ReplRestore` | Scepter → Cosmos | `{ path }` |
| `UnixMethod::BridgeCall` | Cosmos → Scepter | `{ tool_name, parameters }` |
| `UnixMethod::BridgeListTools` | Cosmos → Scepter | Нет |

### Формат Ответа

```json
{
  "success": true,
  "data": { ... },
  "error": null
}
```

## V. Жизненный Цикл Контейнера

```mermaid
stateDiagram-v2
    [*] --> Pending: Цепочка Навыков Запущена
    Pending --> Creating: SnowflakeManager.create()
    Creating --> Starting: Контейнер Запущен
    Starting --> Connected: CosmosConnector.connect()
    Connected --> Ready: Проверка Здоровья Пройдена

    Ready --> Executing: invoke_tool()
    Executing --> Ready: Результат Возвращён

    Ready --> Stopping: Цепочка Навыков Завершена
    Stopping --> Disconnected: CosmosConnector.disconnect()
    Disconnected --> [*]

    Creating --> Failed: Таймаут
    Starting --> Failed: Ошибка Соединения
    Failed --> [*]
```

### Агенты Контейнера

Внутри контейнеров Cosmos только skemma работает локально (движок Boa JS). Все остальные инструменты агентов маршрутизируются через мост HapLotes обратно в Scepter:

| Агент | Роль | В Cosmos? |
| --- | --- | --- |
| SkeMma | Выполнение скриптов (Boa JS) | **Локально** (в процессе) |
| Aporia | Чат LLM | Через мост → Scepter |
| KaLos | Файловый В/В | Через мост → Scepter |
| NeiKos | Управление контейнерами | Через мост → Scepter |
| EleOs | Веб-поиск | Через мост → Scepter |
| Все остальные | Разное | Через мост → Scepter |

## VI. Интеграция Строки Статистики

### Формат Отображения

В `AgentDetailPage` TUI строка статистики показывает:

```mermaid
flowchart LR
    BORDER["|"] --> TOK["1.2k токенов"] --> SEP1["|"] --> DUR["3.5с"] --> SEP2["|"] --> COSMOS["cosmos#042"] --> TIER["[T2]"]

    TOK -.->|"McpToolResult.token_usage"| SRC1["Использование Токенов"]
    DUR -.->|"Instant::now()"| SRC2["Длительность"]
    COSMOS -.->|"AgentIdManager"| SRC3["Номер Агента"]
    TIER -.->|"McpToolConfig.tier"| SRC4["Уровень Модели"]
```

| Сегмент | Источник |
| --- | --- |
| `1.2k токенов` | `McpToolResult.token_usage` |
| `3.5с` | Длительность от `Instant::now()` |
| `cosmos#042` | Номер агента от `AgentIdManager` |
| `[T2]` | Уровень модели от `McpToolConfig.tier` |

## VII. Обработка Ошибок

### Режимы Сбоев

```mermaid
flowchart TD
    Call[Вызов Инструмента] --> Q1{Контейнер Онлайн?}
    Q1 -->|Нет| E1[Ошибка: AGENT_OFFLINE]
    Q1 -->|Да| Q2{Сокет Подключён?}
    Q2 -->|Нет| E2[Ошибка: Соединение Потеряно]
    Q2 -->|Да| Q3{Инструмент Существует?}
    Q3 -->|Нет| E3[Ошибка: Инструмент Не Найден]
    Q3 -->|Да| Q4{Выполнение Успешно?}
    Q4 -->|Нет| E4[Ошибка: Выполнение Не Удалось]
    Q4 -->|Да| Result[Вернуть Результат]

    E1 --> Fallback[Резерв: Попробовать выполнение Scepter]
    E2 --> Retry[Повтор: Переподключить сокет]
```

### Плавная Деградация

Когда контейнер недоступен, система может опционально переключиться на локальное выполнение `Scepter`, если для инструмента зарегистрирована локальная реализация.

## VIII. Будущие Расширения

| Функция | Описание | Приоритет |
| --- | --- | --- |
| Пул контейнеров | Повторное использование контейнеров между цепочками навыков | Средний |
| Мониторинг здоровья | Периодические проверки здоровья контейнеров | Высокий |
| Ограничения ресурсов | Ограничения CPU/памяти на контейнер | Высокий |
| Мульти-контейнерные инструменты | Инструменты, охватывающие несколько контейнеров | Низкий |
| Миграция контейнеров | Перемещение работающих контейнеров между хостами | Низкий |
