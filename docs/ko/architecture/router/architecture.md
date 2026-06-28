+++
title = "Evernight 아키텍처"
description = """evernight —— 크로스 플랫폼 원격 제어 라이브러리 및 데몬: 모듈 맵, 프로토콜 계층, 연결 모델."""
lang = "ko"
category = "architecture"
subcategory = "router"
+++

# Evernight 아키텍처

> **evernight**는 크로스 플랫폼 원격 제어 라이브러리이자 데몬이다. celestia-island
> 생태계의 필수 하드웨어/프로토콜 능력 브로커로서, 어떤 상위 crate도 물리 디바
> 이스와 직접 통신하지 않는다.

## 개요

| 능력 | 모듈 | Feature |
|---|---|---|
| 화면 캡처(X11/DXGI/CoreGraphics) | `screen` | `screen` |
| WebRTC 화면 스트리밍 | `stream` | `webrtc` |
| SSH 원격 셸 + SFTP | `remote` | `remote-ssh` |
| VNC(RFB) 클라이언트 | `vnc` | `remote-vnc` |
| RDP 클라이언트 | `rdp` | `remote-rdp` |
| 하드웨어 원격측정 | `hardware` | `hardware` |
| 산업 프로토콜 | `protocol` | `protocol` / `s7comm` / `opcua` / `ethercat` |
| 직렬 / Modbus | `serial`, `sensor` | `serial` |
| TCP 터널 + NAT 통과 | `tunnel` | `tunnel` / `upnp` |
| 연결 카탈로그(URI) | `connection`, `connection_chain` | 코어 |
| 암호화 자격증명 금고 | `vault` | `vault` |
| 컨테이너 / K8s / libvirt | `container`, `vm_manager` | `container` / `k8s` / `libvirt` / `vm` |
| API 서버(JSON-RPC) | `api` | `api` |

## 세 가지 다형적 백엔드 trait

- `TerminalBackend` —— 텍스트 터미널의 읽기/쓰기/resize(SSH, 직렬, Docker)
- `ViewportBackend` —— 그래픽 데스크톱의 렌더/입력(VNC, RDP, 로컬 화면)
- `FileBackend` —— 파일 작업의 list/get/put/rm(SFTP, shell, 로컬 FS)

새 전송 추가는 플러그인 삽입만으로 —— 소비 측은 변경 불필요.

## 프로토콜 계층

산업 I/O는 두 trait을 통해 중개된다:

- `ProtocolBackend` —— connect / read / write / ping
- `ProtocolProbe` —— 알려지지 않은 엔드포인트의 프로토콜 자동 식별

```
ProtocolRegistry::auto_detect(transport)  →  ProtocolProbeResult
```

백엔드: Modbus(aoba), S7comm(rust7 + snap7-client), MC Protocol, EtherCAT
(ethercrab), EtherNet/IP + CIP, OPC UA(opcua crate, client + server), CAN
(SocketCAN).

### S7 자가 조직망(auto-provision)

evernight에 날 IP를 주면 자가 조직망한다:

```rust
use evernight::protocol::auto_provision;
let profile = auto_provision("192.168.1.10").await?;
```

프로브 → 연결 → DB 스캔 → 구조 프로브 파이프라인이 `S7DeviceProfile`을 반환하
고, 기호 수동 입력은 제로. 일회성 PLC 준비는
[TIA Portal 사전 준비 가이드](../../guides/router/tia-portal-setup.md) 참조.

## 연결 모델

연결은 URI 타입화되어 카탈로그로 관리된다:

```
ssh://user@host:22          s7://10.0.0.5?rack=0&slot=1
vnc://host:5900             opcua://10.0.0.5:4840
serial:///dev/ttyUSB0?baud=9600
```

`connection_chain`은 타깃을 순서가 있는 홉 체인(일반화된 ProxyJump)으로 해결해
터널링에 쓴다.

## Feature flags

`full`(기본)이 전부를 활성화. 각 능력은 독립적으로 게이트 가능하여 최소 의존성
빌드에 쓸 수 있다 —— 예: `--features s7comm,serial`은 산업 프로토콜 부분집합만
출하한다.
