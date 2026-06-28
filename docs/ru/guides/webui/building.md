+++
title = "Руководство по сборке и разработке"
description = """> Аудитория: Контрибьюторы, настраивающие локальную среду разработки shittim-chest."""
lang = "ru"
category = "guides"
subcategory = "webui"
+++

# Руководство по сборке и разработке

> **Аудитория**: Контрибьюторы, настраивающие локальную среду разработки shittim-chest.
> **Последнее обновление**: 2026-05-25

## Предварительные требования

| Инструмент | Минимальная версия | Примечания |
| --- | --- | --- |
| Rust | 1.85+ | Требуется Edition 2024; установка через <https://rustup.rs> |
| Node.js | 20+ | Рекомендуется LTS |
| pnpm | 9+ | `corepack enable && corepack prepare pnpm@latest --activate` |
| just | последняя | Запускатор команд; `cargo install just` |
| PostgreSQL | 18+ | shittim_chest_db для аутентификации + данных чата |
| entelecheia scepter | опционально | Требуется для функций прокси/устройств; опционально для автономного чата |

Проверьте всё:

```bash
rustc --version    # >= 1.85
node --version     # >= 20
pnpm --version     # >= 9
just --version
psql --version     # >= 18
```

## Клонирование и начальная настройка

```bash
git clone https://github.com/celestia-island/shittim-chest.git
cd shittim-chest
cp .env.example .env
```

## Переменные окружения

Отредактируйте `.env` после клонирования. Каждая переменная документирована в файле; ниже приведена сводка.

### Сервер

| Переменная | По умолчанию | Назначение |
| --- | --- | --- |
| `SHITTIM_CHEST_HOST` | `0.0.0.0` | Адрес прослушивания |
| `SHITTIM_CHEST_PORT` | `80` | Порт прослушивания |

### База данных

| Переменная | По умолчанию | Назначение |
| --- | --- | --- |
| `SHITTIM_CHEST_DATABASE_URL` | `postgresql://sc:pass@localhost:5432/shittim_chest` | Строка подключения PostgreSQL |
| `SHITTIM_CHEST_DATABASE_MAX_CONNECTIONS` | `10` | Размер пула соединений SeaORM |

Создайте базу данных и пользователя:

```sql
CREATE USER sc WITH PASSWORD 'pass';
CREATE DATABASE shittim_chest OWNER sc;
```

### JWT и шифрование

| Переменная | По умолчанию | Назначение |
| --- | --- | --- |
| `JWT_SECRET` | `change-me-in-production` | Общий секрет с scepter; **должен совпадать** |
| `JWT_EXPIRATION_SECONDS` | `3600` | Время жизни токена доступа (1 час) |
| `JWT_REFRESH_EXPIRATION_SECONDS` | `604800` | Время жизни токена обновления (7 дней) |
| `SHITTIM_CHEST_ENCRYPTION_KEY` | `change-me-32-bytes-base64-encoded` | Ключ AES-256-GCM для API-ключей и токенов OAuth |

Сгенерируйте ключ для продакшена:

```bash
openssl rand -base64 32
```

### Провайдеры LLM (для автономной работы)

Установите их для использования shittim-chest независимо без entelecheia:

| Переменная | Назначение |
| --- | --- |
| `LLM_DEFAULT_PROVIDER_ENDPOINT` | Конечная точка OpenAI-совместимого API (напр. `https://api.deepseek.com/v1`) |
| `LLM_DEFAULT_PROVIDER_API_KEY` | API-ключ для провайдера |
| `LLM_DEFAULT_PROVIDER_MODELS` | Список моделей через запятую (напр. `deepseek-chat,deepseek-reasoner`) |
| `LLM_DEFAULT_PROVIDER_CATEGORY` | Категория провайдера: `chat` или `image` |
| `LLM_STREAM_BUFFER_SECONDS` | Таймаут буфера потока (по умолчанию: 60) |
| `LLM_MAX_TOKENS_DEFAULT` | Максимум токенов по умолчанию (по умолчанию: 4096) |
| `LLM_REQUEST_TIMEOUT_SECONDS` | Таймаут HTTP-запроса (по умолчанию: 120) |

### Удалённые устройства

| Переменная | По умолчанию | Назначение |
| --- | --- | --- |
| `REMOTE_DEVICES_ENABLED` | `false` | Включить функции удалённых устройств |
| `REMOTE_DEVICES_SCEPTER_SOCK` | `/run/entelecheia/device_stream.sock` | Unix-сокет для данных устройств |
| `REMOTE_DEVICES_FRAME_BUFFER_SIZE` | `4194304` | Размер буфера кадров в байтах |
| `REMOTE_DEVICES_MAX_SESSIONS_PER_USER` | `3` | Макс. одновременных сессий устройств |
| `WEBRTC_ICE_SERVERS` | `stun:stun.l.google.com:19302` | Список ICE-серверов |

### GitHub OAuth

| Переменная | Назначение |
| --- | --- |
| `GITHUB_CLIENT_ID` | Client ID приложения GitHub OAuth |
| `GITHUB_CLIENT_SECRET` | Client secret приложения GitHub OAuth |
| `GITHUB_REDIRECT_URI` | URL обратного вызова OAuth (напр. `https://your-domain/api/auth/github/callback`) |

### Подключение к Scepter (для функций прокси)

| Переменная | По умолчанию | Назначение |
| --- | --- | --- |
| `ENTELECHEIA_SCEPTER_URL` | `http://localhost:8424` | HTTP-конечная точка для scepter |
| `ENTELECHEIA_SCEPTER_WS_URL` | `ws://localhost:8424` | WebSocket-конечная точка для scepter |
| `ENTELECHEIA_TUI_SOCK` | `/run/entelecheia/entelecheia.sock` | Unix-сокет для пересылки триггеров |

### Вебхуки

| Переменная | Назначение |
| --- | --- |
| `WEBHOOK_GITHUB_SECRET` | Секрет HMAC для проверки вебхуков GitHub |
| `WEBHOOK_GITLAB_SECRET` | Токен для проверки вебхуков GitLab |
| `WEBHOOK_PUBLIC_URL` | Публичный URL для конечных точек вебхуков |

## Настройка базы данных

```bash
just db-init      # Создать схему (выполняет миграции SeaORM)
just db-migrate   # Применить ожидающие миграции
```

### Обзор схемы

`shittim_chest_db` владеет данными, обращёнными к пользователю:

| Таблица | Назначение |
| --- | --- |
| `auth_users` | Учётные записи пользователей с хешами паролей argon2 |
| `sessions` | Активные сессии с токенами обновления |
| `api_keys` | Записи API-ключей (хешированные) |
| `oauth_connections` | Привязки сторонних OAuth (GitHub) |
| `conversations` | Диалоги чата |
| `messages` | Сообщения чата с данными вызовов инструментов |
| `llm_providers` | Конфигурации провайдеров LLM (API-ключи зашифрованы) |
| `remote_devices` | Записи удалённых устройств |
| `device_sessions` | Активные сессии устройств |
| `channel_configs` | Конфигурации мультиплатформенных каналов |
| `channel_messages` | Записи сообщений каналов |
| `channel_pairings` | Сопряжения канал-чат |

Сбросить базу данных:

```bash
just db-reset
```

## Разработка бэкенда

```bash
just dev-backend
```

Запускает `cargo run --package shittim_chest`. Сервер стартует на `:80`.

### Команды CLI

```bash
shittim_chest db-init      # Создать схему базы данных
shittim_chest db-migrate   # Применить ожидающие миграции
shittim_chest db-reset     # Удалить и пересоздать схему
shittim_chest server       # Запустить веб-сервер
```

### Горячая перезагрузка

```bash
cargo install cargo-watch
cargo watch -x 'run --package shittim_chest -- server'
```

### Обзор конечных точек API

| Группа маршрутов | Назначение |
| --- | --- |
| `/api/auth/*` | Вход, регистрация, GitHub OAuth, обновление, выход |
| `/api/chat/*` | Диалоги, сообщения, SSE/WS поток, поиск, экспорт |
| `/api/providers/*` | CRUD провайдеров LLM, управление API-ключами, тестирование |
| `/api/generation/*` | Генерация изображений, листинг моделей |
| `/api/devices/*` | Листинг удалённых устройств, сессии, сигнализация WebRTC |
| `/api/webhook/*` | Вход вебхуков GitHub/GitLab/Gitee/своих |
| `/api/proxy/*` | Обратный прокси к scepter (HTTP + WebSocket) |
| `/static/*` | Хостинг статических файлов SPA |

## Разработка фронтенда

### Установка зависимостей

```bash
pnpm install
```

### webui

```bash
just dev    # собрать фронтенд + запустить бэкенд на :3000
just watch  # авто-пересборка при изменениях файлов
```

Оба фронтенда собираются Vite в `dist/`. Бэкенд обслуживает эти статические файлы напрямую на `:3000` — отдельный dev-сервер Vite или прокси не нужен.

## Межпроектная настройка

Для локальной разработки с общим крейтом протокола `arona` пропатчите его на ваш локальный checkout. Отредактируйте `~/.cargo/config.toml` (никогда не коммитьте в репозиторий):

```toml
[patch.'https://github.com/celestia-island/arona']
arona = { path = "/path/to/arona" }
```

Для npm webui использует привязки TS крейта `arona` через псевдоним пути `@celestia-island/arona`, указывающий на `packages/webui/src/types/arona/`.

## Сборка для продакшена

```bash
just build
```

Запускает `cargo build --release` и `pnpm run build:all`. Расположение вывода:

- Бинарник бэкенда: `target/release/shittim_chest`
- Ресурсы фронтенда: `packages/webui/dist/`

### Docker

Сборка и запуск с обёрткой CLI (использует Docker API напрямую):

```bash
just dev
```

Или вручную:

```bash
just build        # собрать образ Docker
just up           # запустить все сервисы
just migrate      # выполнить миграции базы данных
```

Продакшен-бинарник обслуживает ресурсы фронтенда через промежуточное ПО статических файлов Axum по `/`. Отдельный сервер фронтенда не нужен.

## Распространённые проблемы

### Отказ в соединении с базой данных

```text
error: connection to server at "localhost", port 5432 failed
```

**Исправление**: Убедитесь, что PostgreSQL работает и `SHITTIM_CHEST_DATABASE_URL` в `.env` соответствует вашей настройке. Проверьте с помощью `psql "$SHITTIM_CHEST_DATABASE_URL" -c 'SELECT 1'`.

### Scepter недоступен

```text
error: error sending request for url (http://localhost:8424/...)
```

**Исправление**: Запустите экземпляр entelecheia scepter или используйте автономный режим с настроенными провайдерами LLM. Бэкенд работает без scepter для чата/генерации изображений.

### Ошибки CORS в браузере

```text
Access-Control-Allow-Origin header is present on the requested resource
```

**Исправление**: Dev-бэкенд включает CORS для источников `localhost`. Если вы изменили порты, обновите конфигурацию CORS. Продакшен-развёртывания должны настраивать обратный прокси (nginx/caddy) для обработки CORS.

### Сбой pnpm install

**Исправление**: Убедитесь, что используете pnpm 9+. Запустите `corepack enable && corepack prepare pnpm@latest --activate` для настройки правильной версии.

### Сбой cargo build на общих крейтах

**Исправление**: Если у вас есть локальные патчи в `~/.cargo/config.toml`, убедитесь, что пути существуют и имена крейтов совпадают. Удалите раздел патча, чтобы вместо этого использовать git-зависимости.
