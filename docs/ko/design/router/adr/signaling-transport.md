# 시그널링 전송 — Unix 소켓 / TCP 이중 전송

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

시그널링 클라이언트는 WebRTC SDP offer/answer와 ICE candidate를 교환하기 위해 릴레이 서버에 연결한다. 원래는 Unix 도메인 소켓(`tokio::net::UnixStream`)만 사용했으나, 이는 Windows에서 사용할 수 없다. 크로스 플랫폼 지원을 위해서는 폴백이 필요하다.

## 결정

Unix 도메인 소켓을 먼저 시도하고(지원하는 플랫폼에서, `/`로 시작하거나 `.sock`으로 끝나는 경로 형식으로 감지), 실패 시 TCP로 폴백하는 이중 전송 `SignalingStream` 열거형을 구현한다. `TransportWriter` 어댑터는 두 전송에 대해 `AsyncWrite`를 추상화한다. 비Unix 플랫폼에서는 TCP만 사용 가능하다.

## 결과

### 긍정적 측면

- 크로스 플랫폼 시그널링; 동일한 JSON-RPC 프로토콜이 두 전송에서 모두 작동

### 부정적 측면

- TCP 시그널링은 기본적으로 암호화되지 않음
- Windows 사용자는 루프백 전용 바인딩을 보장해야 함
