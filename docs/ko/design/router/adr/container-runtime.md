# 컨테이너 런타임 클라이언트 (Docker/Podman)

- **상태**: Accepted
- **날짜**: 2026-06-09
- **작성자**: Evernight Core Team

## 배경

컨테이너(Docker, Podman) 관리는 범용 연결 관리자의 핵심 사용 사례이다. 작업에는 컨테이너 목록 조회, exec 셸, 로그 보기, 포트 포워딩이 포함된다.

## 결정

Unix 소켓(또는 Windows의 named pipe)을 통해 Docker Engine API를 사용한다. Podman의 Docker 호환 API도 동일한 코드 경로로 지원된다. 타입이 지정된 컨테이너 모델(`ContainerInfo`, `ContainerState`, `ContainerPort`)을 정의한다. 향후: 대화형 컨테이너 셸을 위해 Docker exec attach 위에 `TerminalBackend`를 구현한다.

## 결과

### 긍정적 측면

- Docker API는 문서화가 잘 되어 있고 안정적이며, Podman 호환성이 무료로 제공된다.

### 부정적 측면

- Docker/Podman 데몬이 실행 중이어야 한다.
- Docker 버전 간 API 차이가 존재한다.
