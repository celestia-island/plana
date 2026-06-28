+++
title = "Руководство по использованию CLI"
description = """`entelecheia-cli` — интерфейс командной строки платформы мультиагентного взаимодействия Entelecheia (玄枢). Взаимодействует с сервером scepter через Unix socket JSON-RPC, предоставляя функции чата, управления жизненным циклом служб, контроля агентов, конфигурации и другие возможности."""
lang = "ru"
category = "guides"
subcategory = "core"
+++

# Руководство по использованию CLI

`entelecheia-cli` — интерфейс командной строки платформы мультиагентного взаимодействия Entelecheia (玄枢). Взаимодействует с сервером scepter через Unix socket JSON-RPC, предоставляя функции чата, управления жизненным циклом служб, контроля агентов, конфигурации и другие возможности.

> Примечание: CLI в настоящее время не обладает полным паритетом функций с TUI. Текущее состояние см. в [ARCHITECTURE.md](../../ARCHITECTURE.md).

---

## Содержание

- [Установка](#установка)
- [Основное использование](#основное-использование)
- [Глобальные параметры](#глобальные-параметры)
- [Команды чата](#команды-чата)
- [Управление агентами](#управление-агентами)
- [Жизненный цикл служб](#жизненный-цикл-служб)
- [Конфигурация](#конфигурация)
- [Контекст соединения](#контекст-соединения)
- [Состояние и мониторинг](#состояние-и-мониторинг)
- [Подписки (Layer3)](#подписки-layer3)
- [Запуск агентов](#запуск-агентов)
- [Таймлайн](#таймлайн)
- [Docker-образы](#docker-образы)
- [Продвинутое использование](#продвинутое-использование)

---

## Установка

### Сборка из исходников

```bash
# Клонировать репозиторий
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# Собрать бинарный файл CLI
cargo build --package entelecheia-cli

# Или с помощью just
just cli
```

Бинарный файл находится в `target/debug/entelecheia-cli` (debug) или `target/release/entelecheia-cli` (release).

### Предсобранные бинарные файлы

Предсобранные бинарные файлы доступны на [GitHub Releases](https://github.com/celestia-island/entelecheia/releases). Скачайте архив, подходящий для вашей платформы, и поместите бинарный файл в `PATH`.

---

## Основное использование

```bash
# Показать справку
entelecheia-cli --help

# Отправить сообщение через цепочку навыков
entelecheia-cli send Объясни архитектуру этого проекта

# Отправить сообщение через пайп
echo "Суммируй этот файл" | entelecheia-cli send

# Проверить состояние системы
entelecheia-cli status
```

---

## Глобальные параметры

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | Уровень логирования (trace, debug, info, warn, error) | `warn` |
| `-d, --daemon` | Отправить команду в фоновом режиме и немедленно выйти | — |
| `-c, --clean` | Очистить контейнеры Cosmos и файлы socket | — |
| `-a, --auto-approve` | Автоматически одобрять операции (убедиться, что сервер запущен) | — |
| `-t, --table` | Человекочитаемый табличный вывод (формат ANSI) | По умолчанию |
| `-j, --json` | Вывод в формате JSON (машиночитаемый) | — |
| `-r, --raw` | Чистый текстовый вывод (без форматирования) | — |
| `--format <FORMAT>` | Формат вывода (table, json, raw) | `table` |

Параметры формата вывода:

- `table` — человекочитаемый табличный вывод
- `json` — машиночитаемый вывод в формате JSON

**Примеры:**

```bash
# Очистить контейнеры
entelecheia-cli --clean

# Получить состояние в формате JSON
entelecheia-cli status --format json

# Отправить сообщение в режиме отладки
entelecheia-cli -l debug send "Отладка проблемы подключения"

# Запустить агента в фоновом режиме (немедленный возврат)
entelecheia-cli -d run my-agent --ci
```

---

## Команды чата

Подкоманда `chat` управляет диалоговым взаимодействием с системой агентов сессии.

### Отправка сообщения

```bash
entelecheia-cli chat send [OPTIONS]
```

| Параметр | Описание |
| --- | --- |
| `-m, --message <MSG>` | Текст отправляемого сообщения |
| `--stdin` | Читать сообщение из стандартного ввода |
| `-f, --file <PATH>` | Читать сообщение из файла |

Можно использовать только один источник ввода за раз.

**Примеры:**

```bash
# Отправить сообщение напрямую
entelecheia-cli chat send -m "Привет, что ты можешь делать?"

# Из стандартного ввода
echo "Проанализируй код в src/main.rs" | entelecheia-cli chat send --stdin

# Из файла
entelecheia-cli chat send -f ./prompts/review.txt
```

Команда `chat send` передаёт сообщение через **цепочку навыков** — основной конвейер выполнения, координирующий несколько агентов. Во время выполнения отображается прогресс с помощью вращающейся анимации.

### История чата

```bash
entelecheia-cli chat history [OPTIONS]
```

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--conversation <ID>` | Фильтр по ID беседы | — |
| `--agent <TYPE>` | Фильтр по типу агента | — |
| `--role <ROLE>` | Фильтр по роли (user/assistant/system) | — |
| `--from <ISO8601>` | Дата и время начала (ISO 8601) | — |
| `--to <ISO8601>` | Дата и время окончания (ISO 8601) | — |
| `--limit <N>` | Максимальное количество возвращаемых сообщений | `50` |
| `--offset <N>` | Смещение для пагинации | `0` |

**Примеры:**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### Последние сообщения

```bash
entelecheia-cli chat recent [OPTIONS]
```

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--timeline <ID>` | Фильтр по ID таймлайна/сессии | — |
| `--agent <TYPE>` | Фильтр по типу агента | — |
| `--limit <N>` | Максимальное количество возвращаемых сообщений | `20` |

---

## Управление агентами

Управление жизненным циклом агентов (список, запуск, остановка, перезапуск).

```bash
entelecheia-cli agent <COMMAND>
```

### Команды

```bash
# Список всех агентов и их состояний
entelecheia-cli agent list

# Запустить агента по типу
entelecheia-cli agent start <AGENT_TYPE>

# Остановить запущенного агента
entelecheia-cli agent stop <AGENT_TYPE>

# Перезапустить агента
entelecheia-cli agent restart <AGENT_TYPE>
```

**Доступные типы агентов:** ApoRia, EleOs, EpieiKeia, Haplotes, HubRis, Kalos, NeiKos, OreXis, PhiLia, Polemos, SkeMma, SkoPeo.

> Примечание: агенты работают как библиотечные крейты внутри времени выполнения scepter, а не как отдельные исполняемые файлы. Команда `agent start` пытается создать бинарный файл, соответствующий имени агента, что в основном применимо, когда агент скомпилирован как отдельный бинарный файл. На практике агенты активируются через сервер scepter.

---

## Жизненный цикл служб

Управление стеком служб Entelecheia (玄枢) с использованием контейнеров Docker.

### Инициализация служб

```bash
entelecheia-cli init [OPTIONS]
```

Настраивает полный стек служб: PostgreSQL (с pgvector), Docker registry, сервер scepter и WebUI. Создаёт необходимые сети Docker и загружает/собирает образы.

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--prefix <STR>` | Префикс имени контейнера | `e-` |
| `--source-build` | Собирать образы из исходников вместо загрузки | `false` |
| `--webui-port <PORT>` | Порт WebUI | `3424` |

**Примеры:**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### Запуск всех служб

```bash
entelecheia-cli serve [OPTIONS]
```

Запускает все ранее инициализированные контейнеры. Требуется предварительный вызов `init`.

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--prefix <STR>` | Префикс имени контейнера | `e-` |
| `--webui-port <PORT>` | Порт WebUI | `3424` |

### Остановка всех служб

```bash
entelecheia-cli stop [OPTIONS]
```

Последовательно останавливает все запущенные контейнеры: webui → scepter → registry → postgres.

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--prefix <STR>` | Префикс имени контейнера | `e-` |

### Запуск только WebUI

```bash
entelecheia-cli webui [OPTIONS]
```

Запускает или создаёт только контейнер WebUI.

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--prefix <STR>` | Префикс имени контейнера | `e-` |
| `--webui-port <PORT>` | Порт WebUI | `3424` |

---

## Конфигурация

Просмотр и проверка системной конфигурации.

### Показать конфигурацию

```bash
entelecheia-cli config show
```

Отображает текущую конфигурацию, включая:

- URL базы данных и настройки подключения
- Конфигурацию провайдеров LLM ApoRia (название, модель, эндпоинт)
- Адрес привязки WebSocket
- Уровень логирования

API-ключи в выводе скрыты (отображаются как `***`).

### Проверить конфигурацию

```bash
entelecheia-cli config validate
```

Выполняет проверки:

- URL базы данных установлен
- Настроен хотя бы один провайдер ApoRia с полными настройками
- Адрес привязки WebSocket установлен

Возвращает результат пройдено/не пройдено с подробностями о любых проблемах.

**Пример вывода:**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## Контекст соединения

Подкоманда `context` используется для управления именованными профилями подключения, позволяя переключаться между локальным (Unix socket) и удалённым (WebSocket) серверами scepter. Использование аналогично команде `docker context`.

### Концепция

**Контекст** — это именованный профиль конфигурации, определяющий, как CLI подключается к серверу scepter:

- **local** — подключение через Unix socket (по умолчанию, автоматически разрешается в `/run/.../entelecheia-tui.sock`)
- **remote** — подключение через WebSocket с аутентификацией Bearer token

Контексты хранятся в `~/.config/entelecheia/contexts/contexts.toml`.

### Список контекстов

```bash
entelecheia-cli context list
```

Текущий активный контекст отмечен символом `*`.

### Показать текущий контекст

```bash
entelecheia-cli context show
```

Отображает тип активного контекста, путь socket, URL WS и описание.

### Создать контекст

```bash
# Удалённый контекст WebSocket
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Staging server"

# Дополнительный локальный контекст
entelecheia-cli context create dev --description "Development server"
```

Получение Bearer token от удалённого сервера:

```bash
# На серверной машине
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### Переключение контекста

```bash
entelecheia-cli context use staging
# После этого все команды (send, status, chat и т.д.) будут маршрутизироваться через staging-подключение
```

### Удаление контекста

```bash
entelecheia-cli context remove staging
```

Контекст `default` не может быть удалён.

### Пример рабочего процесса

```bash
# Просмотреть текущий контекст
entelecheia-cli context list

# Создать удалённый контекст для сервера предрелиза
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# Переключиться на среду предрелиза
entelecheia-cli context use staging

# Отправить сообщение через удалённый сервер
entelecheia-cli send "Показать текущие задачи"

# Проверить состояние удалённого сервера
entelecheia-cli status

# Переключиться обратно на локальный
entelecheia-cli context use default
```

---

## Состояние и мониторинг

### Состояние системы

```bash
entelecheia-cli status
```

Отображает:

- Версию сервера
- Состояние подключения (состояние socket)
- Сводку провайдеров LLM
- Адрес привязки WebSocket
- Список агентов с состоянием запущен/остановлен
- Системные ресурсы (использование памяти, средняя нагрузка)

### Запрос пути состояния

Команда `status` принимает параметры в виде пути для запроса конкретных подсистем. Синтаксис поддерживает таймлайны с областью действия агента, проверку истории чата и перечисление устройств.

```bash
entelecheia-cli status <PATH> [--raw]
```

| Синтаксис пути | Описание |
| --- | --- |
| `timeline.#agent[-N]` | Показать последние N записей вызова навыков агента |
| `timeline.#agent[N][M]` | Показать M-й вызов MCP/инструмента в N-м вызове навыка |
| `history[-N]` | Показать последние N сообщений чата (все роли) |
| `history[-N].body` | Показать тело N-го с конца сообщения |
| `device` | Список всех периферийных устройств, распознанных Polemos |
| `device[N]` | Показать подробную информацию об N-м устройстве Polemos |

**Примеры:**

```bash
# История последних 30 вызовов навыков агента Haplotes #001
entelecheia-cli status timeline.#hap_lotes.001[-30]

# Второй вызов MCP/инструмента в третьем вызове навыка
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# Последние 30 сообщений
entelecheia-cli status history[-30]

# Тело третьего с конца сообщения (чистый текст)
entelecheia-cli status history[-3].body --raw

# Все устройства Polemos
entelecheia-cli status device

# Подробности третьего устройства Polemos
entelecheia-cli status device[3]
```

> **Примечание для Shell:** в bash/zsh используйте одинарные кавычки для путей, содержащих `[...]`, чтобы предотвратить glob-раскрытие: `entelecheia-cli status 'history[-30]'`. Символ `#` внутри слова не требует экранирования. В fish shell все вышеуказанные пути не требуют кавычек.

Запросы пути состояния взаимодействуют с сервером через Unix socket JSON-RPC. Запросы `timeline.*` и `history.*` требуют работающего сервера. Запросы `device` требуют регистрации рабочей области Polemos на сервере.

### Просмотр логов

```bash
entelecheia-cli logs [OPTIONS]
```

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `-a, --agent <NAME>` | Фильтр логов по имени агента | Все агенты |
| `-l, --lines <N>` | Количество отображаемых строк (с конца) | `100` |

**Примеры:**

```bash
# Показать последние 200 строк логов всех агентов
entelecheia-cli logs -l 200

# Показать логи ApoRia
entelecheia-cli logs -a ApoRia
```

Логи читаются из каталога `./logs/`. Каждый агент имеет свой собственный файл лога (`ApoRia.log`, `EleOs.log` и т.д.).

---

## Подписки (Layer3)

Управление подписками агентов Layer3 — внешними пакетами агентов, которые можно устанавливать и запускать.

### Список подписок

```bash
entelecheia-cli subscribe list
```

Отображает все настроенные подписки, включая состояние (установлено/ожидает), статус включения, настройки автообновления и источник.

### Добавление подписки

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| Параметр | Описание |
| --- | --- |
| `--name <NAME>` | Название подписки (обязательно) |
| `--source <SOURCE>` | Тип источника: `official`, `github` или `url` (обязательно) |
| `--repository <REPO>` | Репозиторий GitHub (для источника github) |
| `--url <URL>` | Прямой URL (для источника url) |
| `--version <VER>` | Ограничение версии |
| `--auto-update` | Включить автоматическое обновление |
| `--disabled` | Добавить как отключённую |

**Примеры:**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### Удаление подписки

```bash
entelecheia-cli subscribe remove <NAME>
```

### Синхронизация подписок

```bash
# Синхронизировать все подписки
entelecheia-cli subscribe sync

# Синхронизировать определённую подписку
entelecheia-cli subscribe sync --name my-agent
```

### Автообновление

```bash
entelecheia-cli subscribe auto-update
```

Обновляет все подписки с включённым `auto_update`.

---

## Запуск агентов

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

Запускает скрипт агента Layer3. Ищет `.amphoreus/<AGENT>/run.py` в текущем каталоге. При первом выполнении запускает предварительный аудит.

| Параметр | Описание |
| --- | --- |
| `--ci` | Включить режим CI |
| `--auto-pr` | Включить режим автоматического PR |
| `--dry-run` | Пробный запуск (без фактических изменений) |
| `--providers <LIST>` | Список провайдеров через запятую |
| `--output-dir <DIR>` | Выходной каталог |

**Примеры:**

```bash
# Запустить агента Layer3 в режиме пробного запуска
entelecheia-cli run my-agent --dry-run

# Запустить с указанными провайдерами
entelecheia-cli run my-agent --providers openai,anthropic

# Режим CI с автоматическим созданием PR
entelecheia-cli run my-agent --ci --auto-pr

# Запуск в фоновом режиме (немедленный возврат, дочерний процесс выполняется в фоне)
entelecheia-cli -d run my-agent --ci --auto-pr
```

### Фоновый режим (`-d` / `--daemon`)

Флаг фонового режима заставляет CLI повторно создать отсоединённый дочерний процесс, удалив параметр `--daemon`, и немедленно вернуться. Дочерний процесс наследует исходную команду и выполняется независимо. После этого можно использовать `status` для отслеживания прогресса.

Подходит для длительных операций, таких как `run`, `init`, `deploy`:

```bash
# Отправить запуск агента в фон
entelecheia-cli -d run my-agent

# Отправить инициализацию служб в фон
entelecheia-cli -d init --prefix prod-

# Проверить состояние позже
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## Таймлайн

Просмотр таймлайнов сессий.

### Список таймлайнов

```bash
entelecheia-cli timeline list [OPTIONS]
```

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--agent <TYPE>` | Фильтр по типу агента | — |
| `--limit <N>` | Максимальное количество результатов | `50` |
| `--offset <N>` | Смещение для пагинации | `0` |

### Показать детали таймлайна

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| Параметр | Описание | По умолчанию |
| --- | --- | --- |
| `--include-messages` | Включить сообщения в вывод | `true` |

---

## Docker-образы

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

Собирает или загружает Docker-образы, необходимые платформе.

| Параметр | Описание |
| --- | --- |
| `--source-build` | Собирать образы из исходников вместо загрузки |
| `--tag <TAG>` | Тег образа (по умолчанию: `latest`) |

**Примеры:**

```bash
# Собрать все образы из исходников
entelecheia-cli init-docker-images --source-build

# Загрузить с пользовательским тегом
entelecheia-cli init-docker-images --tag v0.2.0
```

Управляемые образы:

- `entelecheia` — сервер оркестрации (со встроенной средой выполнения cosmos)
- `pgvector/pgvector` — PostgreSQL с расширением векторного поиска

---

## Продвинутое использование

### Вывод в формате JSON для скриптов

Используйте `--format json` для получения машиночитаемого вывода, который можно передать в `jq` или другие инструменты:

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### Цепочка очистки и инициализации

```bash
# Полный снос и перестройка
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### Режим отладки

```bash
# Включить логирование уровня trace для отладки
entelecheia-cli -l trace send "Тестовое сообщение"
```

### Совместное использование с TUI

CLI и TUI подключаются к одному и тому же серверу scepter. Их можно использовать одновременно:

- Запустите TUI для интерактивных сессий: `cargo run --bin entelecheia-tui`
- Используйте CLI для написания скриптов, автоматизации и быстрых запросов

---

## Устранение неполадок

### "No command specified"

Запустите `--help` для просмотра доступных команд или используйте `send "сообщение"` для быстрой отправки сообщения.

### "Failed to connect to Docker"

Убедитесь, что Docker (или Podman) запущен:

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

Агенты являются внутренними библиотечными крейтами времени выполнения scepter, а не отдельными бинарными файлами. Запустите сервер scepter для активации агентов:

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

Настройте провайдеров ApoRia через переменные окружения. Инструкции по настройке провайдеров см. в [руководстве по сборке](building.md).

### "Configuration validation failed"

Запустите `entelecheia-cli config validate`, чтобы увидеть, какие проверки не пройдены. Распространённые проблемы:

- Отсутствует переменная окружения `DATABASE_URL`
- Неполные настройки провайдера ApoRia (имя, модель, `api_key`)
- Отсутствует адрес привязки WebSocket
