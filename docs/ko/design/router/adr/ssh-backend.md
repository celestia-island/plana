# SSH 백엔드 — russh와 공유 연결

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

Evernight는 원격 셸, 파일 작업, 터미널 접근을 위해 SSH가 필요하다. 초기 코드베이스에는 중복된 연결 로직을 가진 세 개의 개별 SSH 핸들러 구현체(FileHandler, SshHandler, TerminalHandler)가 있었다. 이 프로젝트는 순수 Rust를 요구한다(`Command::new("ssh")` 셸 호출 금지).

## 결정

SSH 백엔드로 `russh`(순수 Rust SSH-2 구현체)를 사용한다. 모든 SSH 핸들러 구현체를 `remote/connection.rs`의 단일 `DefaultSshHandler`로 통합하고, 공유 `connect_session()` 함수를 사용한다. 모든 SSH 작업(셸, 파일, 터미널)은 이 연결 추상화를 공유한다.

## 결과

### 긍정적 측면

- SSH 인증 로직의 단일 진실 공급원
- 추후 연결 풀링을 쉽게 추가 가능

### 부정적 측면

- `russh`가 엣지 케이스에서 OpenSSH보다 뒤처질 수 있음
- SSH 에이전트 포워딩이 아직 내장되지 않음
