+++
title = "TIA Portal 사전 준비 —— evernight 연결"
description = """TIA Portal에서 S7-1200/1500 PLC를 한 번만 설정하여, evernight가 연결·자가 조직망 구성·읽기/쓰기를 이후 사람의 개입 없이 모두 자동으로 수행하도록 하는 절차."""
lang = "ko"
category = "guides"
subcategory = "router"
+++

# TIA Portal 사전 준비 —— evernight 연결

> **목표**: TIA Portal에서 지멘스 S7-1200/1500 PLC를 **한 번만** 설정하여,
> evernight가 연결·자가 조직망 구성·읽기/쓰기를 **이후 모두 자동으로** 수행하
> 도록 한다. 이는 CPU 속성의 일회성 설정이며, 래더/SCL 프로그램 로직은 전혀 건
> 드리지 않는다.

evernight는 **두 채널**로 지멘스 PLC에 연결된다. PLC가 노출하는 것에 따라 선택:

| 채널 | 포트 | 접근 방식 | 필요 TIA 준비 | 적용 |
|------|------|----------|--------------|------|
| **S7comm** | 102 | M / I / Q / DB의 원시 바이트 R/W | PUT/GET + 비최적화 DB | 레거시, 경량, OPC UA 라이선스 불필요 |
| **OPC UA** | 4840 | 기호적, 자기 서술적 | 내장 server 활성화 | **권장** —— 자동 발견, 최적화 DB도 OK |

OPC UA를 켤 수 있으면 우선하라: evernight가 기호 주소 공간 전체를 **자동 브라우
즈**하여, 기호 수동 입력이 제로가 된다.

---

## 경로 A —— S7comm(원시 레지스터 접근)

### A.1 PUT/GET 통신 활성화

S7-1200/1500은 기본적으로 외부 S7 읽기/쓰기를 차단한다.

1. **TIA Portal**에서 프로젝트를 연다.
2. 디바이스/네트워크 뷰에서 **CPU를 클릭**.
3. 속성 → **보호 및 보안(Protection & Security)→ 연결 메커니즘(Connection mechanisms)**.
4. **"원격 파트너로부터의 PUT/GET 통신 접근 허용(Permit access with PUT/GET communication from remote partner)"** 체크.
5. 하드웨어 구성을 CPU에 다운로드.

### A.2 대상 DB를 비최적화로

최적화 블록 액세스(S7-1200/1500 기본값)는 고정 바이트 오프셋이 없어 절대 주소
읽기가 실패한다. evernight가 읽고 쓸 각 DB마다:

1. DB 우클릭 → **속성(Properties)**.
2. **"최적화된 블록 액세스(Optimized block access)"** 체크 해제.
3. 재컴파일 후 다운로드.

> M 마커와 I/Q 프로세스 이미지는 항상 바이트 주소 지정 가능 —— 변경 불필요. 이
> 단계는 DB에만 해당.

### A.3 evernight에서 연결

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500: `rack=0, slot=1`
- S7-300: `slot=2`

코드로 자가 조직망:

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures가 읽기 가능한 각 DB를 기술
```

---

## 경로 B —— OPC UA(권장)

### B.1 펌웨어 및 라이선스 전제

- CPU 펌웨어 **V2.0+**(OPC UA 메서드는 V2.5+).
- CPU 등급에 맞는 **SIMATIC OPC UA 런타임 라이선스**(CPU 속성 →
  **런타임 라이선스(Runtime licenses)→ OPC UA**에서 할당, 컴플라이언스상 필수).

### B.2 OPC UA server 활성화

1. 네트워크/디바이스 뷰에서 **CPU를 클릭**.
2. 속성 → **OPC UA → 일반(General)**: server 이름 입력.
3. 속성 → **OPC UA → 서버(Server)**: **"OPC UA 서버 활성화(Activate OPC UA server)"** 체크.
4. 클라이언트가 도달할 **PROFINET 인터페이스**에 server 할당.

### B.3 기호 변수 노출

- **OPC UA → 서버 → 서버 인터페이스(Server interface)**: **"표준 SIMATIC 서
  버 인터페이스(Standard SIMATIC server interface)"** 선택, 모든 기호 태그/DB
  (최적화 DB 포함)를 자동 게시.

### B.4 인증 및 보안

- **OPC UA → 서버 → 사용자 인증(User authentication)**: 익명(신뢰할 수 있는
  LAN) 또는 사용자 이름/비밀번호.
- **OPC UA → 서버 → 보안(Security)**: 정책 선택. 최초 연결은 `None`이 가장 간
  단. 운영은 `Sign & Encrypt`.
- 펌웨어 **V3.1+ 이자 TIA V19+**에서, **"OPC UA 서버 접근(OPC UA server
  access)"** 기능 역할/런타임 권한을 연결 사용자에게 부여.

### B.5 클라이언트 인증서 신뢰

OPC UA 클라이언트는 연결 시 X.509 인증서를 제시하고, PLC는 알려지지 않은 인증서
를 격리한다. evernight의 첫 연결 후:

1. TIA Portal → **CPU → 인증서(Certificates)**(온라인), **또는**
2. PLC **웹 서버** → "Communication with OPC UA clients", **또는**
3. CPU 디스플레이의 인증서 관리자.

evernight 클라이언트 인증서를 **승인/신뢰**.

### B.6(선택) OPC UA NodeSet XML 내보내기

NodeSet 파일은 모든 변수의 오프라인 지도로, 실연결 없이 사전 계획에 유용:

1. CPU 속성 → **OPC UA → 서버 → 내보내기(Export)**.
2. **"OPC UA XML 파일 내보내기(Export OPC UA XML file)"** 클릭,
   `*.Opc.Ua.NodeSet2.xml` 저장.

### B.7 다운로드

**하드웨어 구성**을 다운로드. 이들은 CPU 속성이지 프로그램 로직이 아니다 —— 래
더/SCL 코드는 무결.

### B.8 evernight에서 연결

엔드포인트 URL:

```
opc.tcp://192.168.1.10:4840
```

evernight는 OPC UA 클라이언트로 연결해, 기호 트리 전체를 **브라우즈**하고 이름으
로 읽고 쓴다 —— 수동 기호 입력 제로, 최적화 DB도 그대로 접근 가능.

---

## 연결성 검증(제로 리스크 프로브)

출력을 구동하기 전에 읽기 전용 프로브로 채널 생존을 확인:

```bash
# 포트 102에서 S7comm을 말하는가?
evernight probe 192.168.1.10 --ports 102

# 포트 4840에서 OPC UA server가 떠 있는가?
evernight probe 192.168.1.10 --ports 4840
```

둘 다 수동적 핸드셰이크 —— 아무것도 읽지 않고 쓰지 않는다.

---

## 안전 경계

- 안전 인터록(비상정지, 리미트, 과부하)을 S7/OPC UA 경로에 **절대** 두지 말 것.
  PLC 스캔 내에 유지. 네트워크 단절로 안전 기능이 무효화되어서는 안 된다.
- evernight 제어는 **저속** 부하(밸브, 상태 기계, 모드 전환)에 적합. S7/OPC UA
  왕복 지연은 약 10–50 ms —— 감독 수준은 충분, 모션/서보는 너무 느리다.
- Q 출력을 직접 쓰기보다, 기존 PLC 로직이 작용하는 **명령 M 비트**를 쓰는 것을
  우선(트리거 원을 가로챈다).

---

## 문제 해결

| 증상 |가능한 원인 | 해결 |
|------|-----------|------|
| S7 연결 거부 / COTP 확인 없음 | PUT/GET 미활성; rack/slot 오류; 방화벽 | A.1; `rack=0 slot=1`(1200/1500) 확인 |
| DB 읽기가 "optimized" / InvalidAddress 반환 | 최적화 블록 액세스 켜짐 | A.2 —— 최적화 해제 후 재컴파일 |
| OPC UA 엔드포인트 도달 불가 | server 미활성; 미다운로드; 라이선스 부족 | B.2 / B.7 / B.1 |
| OPC UA 연결 후 곧 거부 | 클라이언트 인증서 미신뢰 | B.5 |
| 브라우즈 결과 비어 있음 | 표준 SIMATIC 인터페이스 미활성 | B.3 |

---

## 참고

- [OPC UA server 활성화(S7-1500)— STEP 7 V20 문서](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [OPC UA server 접근(엔드포인트 URL / 포트 4840)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [OPC UA XML 파일 내보내기(NodeSet)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
