+++
title = "Чек-лист производственного развёртывания Entelecheia"
description = """> Чек-лист из 12 шагов для развёртывания Entelecheia в production."""
lang = "ru"
category = "design"
subcategory = "core"
+++

# Чек-лист производственного развёртывания Entelecheia

> Чек-лист из 12 шагов для развёртывания Entelecheia в production.

## Предварительная подготовка

- [ ] **1. Выбор режима базы данных**
  - Встроенный pglite: один бинарный файл, без внешней БД. Подходит для <50 одновременных агентов.
  - PostgreSQL: рекомендуется для production. Установите `DATABASE_URL`.

  ```bash
  # Встроенный режим
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # Режим PostgreSQL
  docker-compose up -d
  ```

- [ ] **2. Настройка идентификации пользователя**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

Этот UUID является идентификатором владельца рабочего пространства. Все операции агентов привязаны к нему.

- [ ] **3. Настройка провайдеров LLM**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

Ключи API шифруются в состоянии покоя с помощью AES-256-GCM через агента Aporia.

- [ ] **4. Настройка контейнерной среды выполнения**
  - Docker (по умолчанию): `--container-backend docker`
  - Youki (rootless OCI): `--container-backend youki`
  - Проверьте профиль seccomp: `configs/seccomp/`

- [ ] **5. Проверка политик безопасности**

  ```bash
  # Список зарегистрированных политик безопасности
  entelecheia-cli security policy-list

  # Проверка конфигурации sentinel OreXis
  entelecheia-cli config show orexis
  ```

## Развёртывание

- [ ] **6. Сборка или загрузка образа**

  ```bash
  # Сборка из исходников
  docker build -t entelecheia:latest .

  # Или использование релиза
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. Запуск сервиса**

  ```bash
  # Используя Docker Compose (рекомендуется)
  docker-compose up -d

  # Или автономно
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. Проверка работоспособности**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. Инициализация Docker-образов для агентов**

  ```bash
  entelecheia-cli init-docker-images
  ```

Это собирает образы контейнеров, используемые каждым агентом Layer-1 для изолированного выполнения.

## После развёртывания

- [ ] **10. Настройка мониторинга**

  ```bash
  # Включить трассировку
  export RUST_LOG=info,entelecheia=debug

  # Проверить временную шкалу на наличие проблем
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. Настройка резервного копирования**
  - Встроенный режим: резервное копирование директории `/data`
  - PostgreSQL: `pg_dump` или WAL архивирование
  - Журналы аудита временной шкалы: периодический экспорт

- [ ] **12. Нагрузочное тестирование**

  ```bash
  # Отправить тестовое сообщение
  entelecheia-cli send "Привет, проверка работоспособности системы"

  # Проверить статус агентов
  entelecheia-cli agent list

  # Проверить аудиторский след
  entelecheia-cli trace-chain demiurge.001
  ```

## Усиление безопасности (Рекомендуется)

| Проверка | Команда |
| --- | --- |
| Проверить отсутствие секретов в env | `env \| grep -i key` |
| Проверить группы RBAC | `entelecheia-cli security rbac-list` |
| Проверить ограничения скорости | `entelecheia-cli config show channel.rate_limit` |
| Проверить изоляцию контейнеров | `docker inspect entelecheia \| grep SecurityOpt` |
| Проверить журнал аудита OreXis | `entelecheia-cli logs --agent orexis --lines 100` |

## Устранение неполадок

| Симптом | Диагностика |
| --- | --- |
| Агенты не отвечают | `entelecheia-cli status` → проверить, запущен ли scepter |
| Ошибки вызовов LLM | Проверить ключи API: `entelecheia-cli config show providers` |
| Ошибки контейнеров | `docker logs entelecheia` → искать ошибки Youki/Docker |
| Проблемы с базой данных | Проверить `DATABASE_URL` или права доступа к директории данных pglite |
| Отказ в доступе к инструменту | `entelecheia-cli security policy-list` → проверить отклонённые вызовы |
