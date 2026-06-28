+++
title = "Архитектура Evernight"
description = """evernight — кроссплатформенная библиотека и демон удалённого управления: карта модулей, слой протоколов, модель соединений."""
lang = "ru"
category = "architecture"
subcategory = "router"
+++

# Архитектура Evernight

> **evernight** — кроссплатформенная библиотека и демон удалённого управления.
> Это обязательный брокер аппаратных/протокольных возможностей экосистемы
> celestia-island — ни один вышестоящий crate не общается с физическими
> устройствами напрямую.

## Кратко

| Возможность | Модуль | Feature |
|---|---|---|
| Захват экрана (X11/DXGI/CoreGraphics) | `screen` | `screen` |
| Потоковая передача экрана WebRTC | `stream` | `webrtc` |
| Удалённый SSH-шелл + SFTP | `remote` | `remote-ssh` |
| Клиент VNC (RFB) | `vnc` | `remote-vnc` |
| Клиент RDP | `rdp` | `remote-rdp` |
| Аппаратная телеметрия | `hardware` | `hardware` |
| Промышленные протоколы | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| Последовательный порт / Modbus | `serial`, `sensor` | `serial` |
| TCP-туннели + обход NAT | `tunnel` | `tunnel` / `upnp` |
| Каталог соединений (URI) | `connection`, `connection_chain` | ядро |
| Шифрованное хранилище учётных данных | `vault` | `vault` |
| Контейнеры / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| API-сервер (JSON-RPC) | `api` | `api` |

## Три полиморфных trait-а бэкенда

- `TerminalBackend` — read/write/resize для текстовых терминалов (SSH, serial, Docker)
- `ViewportBackend` — render/input для графического рабочего стола (VNC, RDP, локальный экран)
- `FileBackend` — list/get/put/rm для файлов (SFTP, shell, локальная ФС)

Добавление транспорта — это plug-in — потребители не меняются.

## Слой протоколов

Промышленный ввод-вывод брокерируется через два trait-а:

- `ProtocolBackend` — connect / read / write / ping
- `ProtocolProbe` — автоопределение протокола неизвестной конечной точки

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

Бэкенды: Modbus (aoba), S7comm (rust7 + snap7-client), MC Protocol, EtherCAT
(ethercrab), EtherNet/IP + CIP, OPC UA (crate opcua, клиент + сервер), CAN
(SocketCAN).

### Самоорганизация S7 (auto-provision)

Дайте evernight «голый» IP — и он самоорганизуется:

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

Конвейер probe → connect → scan-DB → structure-probe возвращает
`S7DeviceProfile` без ручного ввода символов. Одноразовую подготовку ПЛК см. в
[руководстве по предварительной подготовке TIA Portal](../../guides/router/tia-portal-setup.md).

## Модель соединений

Соединения типизируются URI и управляются каталогом:

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain` разрешает цель в упорядоченную цепочку переходов
(обобщённый ProxyJump) для туннелирования.

## Feature flags

`full` (по умолчанию) включает всё. Каждая возможность независимо гейтится для
сборок с минимальными зависимостями — напр. `--features s7comm,serial` поставляет
только подмножество промышленных протоколов.
