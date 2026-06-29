+++
title = "Начало работы — Evernight"
description = """Начало работы с evernight — сборка, запуск и первые команды."""
lang = "ru"
category = "guides"
subcategory = "router"
+++

# Начало работы — Evernight

Evernight (长夜月) — кроссплатформенная библиотека и демон удалённого управления, написанные на Rust. Он объединяет захват экрана, потоковую передачу через WebRTC, удалённую оболочку SSH, доступ к удалённому терминалу, передачу файлов, аппаратную телеметрию, поддержку промышленных протоколов (Modbus, S7comm, зондирование OPC-UA) и обход NAT в одном переиспользуемом крейте и автономном CLI-бинарнике.

## Предварительные требования

- Rust 1.85 или новее (издание 2024)
- Компилятор C для вашей платформы (MSVC на Windows, GCC/Clang на Linux/macOS)
- Для аппаратной телеметрии: `nvidia-smi` (GPU NVIDIA), `libudev` на Linux
- Для промышленных протоколов: последовательный порт (`/dev/ttyUSB*`) или сетевой доступ к ПЛК

## Сборка

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

Основной бинарник находится в `target/release/evernight`.

## Быстрый старт

CLI использует подкоманды. Выполните `evernight --help`, чтобы увидеть их все.

### SSH — выполнение удалённой команды

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — операции с файлами

```bash
# Загрузить локальный файл на удалённый узел
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# Скачать удалённый файл
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# Вывести список содержимого удалённого каталога
evernight file ls root@192.168.1.100:/etc/
```

### SSH — SOCKS5-прокси

```bash
# Запустить локальный SOCKS5-прокси (порт 1080), туннелированный через SSH-jump-хост
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### Аппаратная телеметрия

```bash
evernight hw
```

### Зондирование сетевых протоколов

```bash
# Зондировать типичные промышленные порты на узле
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### Опрос промышленных датчиков

```bash
# Опросить датчики по манифесту оборудования и отправлять тревоги в entelecheia
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### Определение типа NAT

```bash
evernight nat
```

### API-сервер (JSON-RPC поверх WebSocket)

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Флаги возможностей

Evernight управляется через feature-флаги, поэтому вы компилируете только то, что нужно:

```toml
[dependencies]
# Минимум: SSH + аппаратная телеметрия
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# Промышленный набор: Modbus + S7comm + поддержка манифестов
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# Всё сразу (по умолчанию)
evernight = { version = "0.1", features = ["full"] }
```

| Возможность | Включает |
|---------|---------|
| `remote-ssh` | SSH exec, передача файлов, терминал, проброс портов, SOCKS5-прокси |
| `remote-vnc` | VNC-клиент (RFB) |
| `remote-rdp` | Каркас транспорта RDP (TPKT/COTP/MCS) |
| `serial` | Последовательный порт + Modbus RTU (через aoba) |
| `s7comm` | Клиент S7comm + загрузка/прошивка блоков (через rust7 + snap7-client) |
| `protocol` | Зондирование протоколов + абстракция-трейт ProtocolBackend |
| `sensor` | Цикл опроса датчиков, оценка тревог, хранение временных рядов |
| `manifest` | Схема манифеста оборудования TOML/JSON + конвертеры времени выполнения |
| `container` | Управление контейнерами Docker / Podman |
| `hardware` | Телеметрия CPU/GPU/памяти/хранилища |
| `screen` | Захват экрана + кодирование JPEG/VP9 |
| `webrtc` | Потоковая передача экрана через WebRTC |
| `tunnel` | Проброс TCP-портов + обход NAT |
| `api` | API-сервер JSON-RPC 2.0 (ws/wss/ipc) |

## Основные возможности

- **Захват экрана** — перечисление дисплеев, захват кадров в формате raw RGBA
- **Потоковая передача через WebRTC** — JPEG поверх DataChannel или видеодорожка VP9; поддержка ICE/STUN
- **Удалённая оболочка SSH** — выполнение команд, передача файлов, открытие терминалов через `russh`
- **Передача файлов** — загрузка/скачивание с обратными вызовами прогресса поверх SSH
- **Аппаратная телеметрия** — CPU, GPU, память, хранилище, PCI-устройства
- **Промышленные протоколы** — Modbus RTU/TCP, S7comm (Siemens), зондирование OPC-UA
- **Опрос датчиков** — декларативный цикл опроса на основе манифеста с маршрутизацией тревог по ISA-18.2
- **TCP-туннелирование** — локальный/удалённый проброс портов + динамический проброс SOCKS5
- **Обнаружение NAT** — определение типа NAT на основе STUN
- **API-сервер** — JSON-RPC 2.0 поверх WebSocket / IPC для веб-фронтендов

## Что дальше

- Ознакомьтесь с **[Руководством по интеграции промышленных протоколов](./protocols.md)**, чтобы узнать об использовании Modbus/S7comm
- См. `evernight <command> --help` для параметров конкретной команды
- Выполните `cargo doc --open` для полного описания API
- Запустите интеграционные тесты, чтобы проверить свою конфигурацию (оборудование не требуется):
```bash
  cargo test --features full --test s7comm_integration    # S7comm против snap7-server
  cargo test --features full --test serial_integration    # Modbus против виртуального последовательного порта
```
