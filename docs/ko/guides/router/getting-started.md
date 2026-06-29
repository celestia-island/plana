+++
title = "빠른 시작 — Evernight"
description = """evernight 시작하기 —— 빌드, 실행, 첫 명령."""
lang = "ko"
category = "guides"
subcategory = "router"
+++

# 빠른 시작 — Evernight

Evernight(长夜月)는 Rust로 작성된 크로스 플랫폼 원격 제어 라이브러리 및 데몬입니다. 화면 캡처, WebRTC 스트리밍, SSH 원격 셸, 원격 터미널, 파일 전송, 하드웨어 원격 측정, 산업 프로토콜 지원(Modbus, S7comm, OPC-UA 프로빙) 및 NAT 통과 기능을 재사용 가능한 크레이트와 독립적인 CLI 바이너리로 통합합니다.

## 사전 요구 사항

- Rust 1.85 이상(2024 에디션)
- 플랫폼에 해당하는 C 컴파일러(Windows는 MSVC, Linux/macOS는 GCC/Clang)
- 하드웨어 원격 측정: `nvidia-smi`(NVIDIA GPU), Linux에서는 `libudev` 필요
- 산업 프로토콜: 직렬 포트(`/dev/ttyUSB*`) 또는 PLC의 네트워크 접근 권한

## 컴파일

```bash
git clone https://github.com/celestia-island/evernight.git
cd evernight
cargo build --release
```

메인 바이너리는 `target/release/evernight`에 있습니다.

## 빠른 시작

CLI는 서브커맨드를 사용합니다. `evernight --help`를 실행하여 모든 명령을 확인하세요.

### SSH — 원격 명령 실행

```bash
evernight exec --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519 \
  --command "uname -a"
```

### SSH — 파일 작업

```bash
# 上传本地文件到远程主机
evernight file cp ./config.yaml root@192.168.1.100:/etc/app/config.yaml

# 下载远程文件
evernight file get root@192.168.1.100:/var/log/syslog ./syslog

# 列出远程目录
evernight file ls root@192.168.1.100:/etc/
```

### SSH — SOCKS5 프록시

```bash
# 通过 SSH 跳板机启动本地 SOCKS5 代理（端口 1080）
evernight proxy 1080 --host 192.168.1.100 --user root --key ~/.ssh/id_ed25519
```

### 하드웨어 원격 측정

```bash
evernight hw
```

### 네트워크 프로토콜 프로빙

```bash
# 探测主机的常见工业端口
evernight probe 192.168.1.20 --ports 502,102,4840,22
```

### 산업 센서 폴링

```bash
# 从硬件 manifest 轮询传感器，向 entelecheia 发送告警
evernight sensor-poll --manifest corridor.toml \
  --entelecheia-socket /run/entelecheia/hardware-events.sock
```

### NAT 유형 감지

```bash
evernight nat
```

### API 서비스(WebSocket 기반 JSON-RPC)

```bash
evernight api-serve --transport ws --host 0.0.0.0 --port 50000
```

## Feature 토글

Evernight은 feature 토글을 사용하여 필요한 부분만 컴파일합니다:

```toml
[dependencies]
# 最小：SSH + 硬件遥测
evernight = { version = "0.1", features = ["remote-ssh", "hardware"] }

# 工业：Modbus + S7comm + manifest 支持
evernight = { version = "0.1", features = ["serial", "s7comm", "manifest"] }

# 全部（默认）
evernight = { version = "0.1", features = ["full"] }
```

| Feature | 활성화 내용 |
|---------|---------|
| `remote-ssh` | SSH 실행, 파일 전송, 터미널, 포트 포워딩, SOCKS5 프록시 |
| `remote-vnc` | VNC(RFB) 클라이언트 |
| `remote-rdp` | RDP 전송 계층 스켈레톤(TPKT/COTP/MCS) |
| `serial` | 직렬 포트 + Modbus RTU(aoba 경유) |
| `s7comm` | S7comm 클라이언트 + 블록 다운로드/플래싱(rust7 + snap7-client 경유) |
| `protocol` | 프로토콜 프로빙 + ProtocolBackend trait 추상화 |
| `sensor` | 센서 폴링 루프, 알림 평가, 시계열 저장 |
| `manifest` | 하드웨어 manifest TOML/JSON 스키마 + 런타임 변환기 |
| `container` | Docker / Podman 컨테이너 관리 |
| `hardware` | CPU/GPU/메모리/스토리지 원격 측정 |
| `screen` | 화면 캡처 + JPEG/VP9 인코딩 |
| `webrtc` | WebRTC 화면 스트리밍 |
| `tunnel` | TCP 포트 포워딩 + NAT 통과 |
| `api` | JSON-RPC 2.0 API 서비스(ws/wss/ipc) |

## 핵심 기능

- **화면 캡처** — 디스플레이 열거, 원시 RGBA 프레임 캡처
- **WebRTC 스트리밍** — DataChannel 상의 JPEG 또는 VP9 비디오 트랙; ICE/STUN 지원
- **SSH 원격 셸** — `russh`를 통해 명령 실행, 파일 전송, 터미널 열기
- **파일 전송** — SSH를 통한 업로드/다운로드, 진행률 콜백 포함
- **하드웨어 원격 측정** — CPU, GPU, 메모리, 스토리지, PCI 장치
- **산업 프로토콜** — Modbus RTU/TCP, S7comm(지멘스), OPC-UA 프로빙
- **센서 폴링** — 선언적 manifest 기반 폴링 루프, ISA-18.2 알림 라우팅 포함
- **TCP 터널** — 로컬/원격 포트 포워딩 + SOCKS5 동적 포워딩
- **NAT 발견** — STUN 기반 NAT 유형 감지
- **API 서비스** — WebSocket/IPC 기반 JSON-RPC 2.0, 웹 프론트엔드용

## 다음 단계

- Modbus/S7comm 사용법은 **[산업 프로토콜 연동 가이드](./protocols.md)**를 참조하세요
- 각 명령 옵션을 보려면 `evernight <명령> --help`를 실행하세요
- 전체 API 참조를 보려면 `cargo doc --open`을 실행하세요
- 환경을 검증하려면 통합 테스트를 실행하세요(하드웨어 불필요):
```bash
  cargo test --features full --test s7comm_integration    # S7comm 对 snap7-server
  cargo test --features full --test serial_integration    # Modbus 对虚拟串口
```
