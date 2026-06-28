+++
title = "Entelecheia 프로덕션 배포 체크리스트"
description = """> Entelecheia를 프로덕션에 배포하기 위한 12단계 체크리스트."""
lang = "ko"
category = "design"
subcategory = "core"
+++

# Entelecheia 프로덕션 배포 체크리스트

> Entelecheia를 프로덕션에 배포하기 위한 12단계 체크리스트.

## 배포 전

- [ ] **1. 데이터베이스 모드 선택**
  - 임베디드 pglite: 단일 바이너리, 외부 DB 불필요. 50개 미만 동시 에이전트에 적합.
  - PostgreSQL: 프로덕션 권장. `DATABASE_URL` 설정.

  ```bash
  # 임베디드 모드
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # PostgreSQL 모드
  docker-compose up -d
  ```

- [ ] **2. 사용자 신원 구성**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

이 UUID는 워크스페이스 소유자 신원입니다. 모든 에이전트 작업은 이에 범위가 지정됩니다.

- [ ] **3. LLM 프로바이더 설정**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

API 키는 Aporia 에이전트를 통해 AES-256-GCM으로 저장 시 암호화됩니다.

- [ ] **4. 컨테이너 런타임 구성**
  - Docker (기본): `--container-backend docker`
  - Youki (루트리스 OCI): `--container-backend youki`
  - seccomp 프로필 확인: `configs/seccomp/`

- [ ] **5. 보안 정책 검토**

  ```bash
  # 등록된 보안 정책 목록
  entelecheia-cli security policy-list

  # OreXis 센티넬 구성 검토
  entelecheia-cli config show orexis
  ```

## 배포

- [ ] **6. 이미지 빌드 또는 풀**

  ```bash
  # 소스에서 빌드
  docker build -t entelecheia:latest .

  # 또는 릴리스 사용
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. 서비스 시작**

  ```bash
  # Docker Compose 사용 (권장)
  docker-compose up -d

  # 또는 독립 실행형
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. 상태 확인**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. 에이전트용 Docker 이미지 초기화**

  ```bash
  entelecheia-cli init-docker-images
  ```

이는 격리된 실행을 위해 각 레이어 1 에이전트가 사용하는 컨테이너 이미지를 빌드합니다.

## 배포 후

- [ ] **10. 모니터링 설정**

  ```bash
  # 추적 활성화
  export RUST_LOG=info,entelecheia=debug

  # 타임라인에서 문제 확인
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. 백업 구성**
  - 임베디드 모드: `/data` 디렉터리 백업
  - PostgreSQL: `pg_dump` 또는 WAL 아카이빙
  - 타임라인 감사 로그: 주기적 익스포트

- [ ] **12. 부하 테스트**

  ```bash
  # 테스트 메시지 전송
  entelecheia-cli send "안녕하세요, 시스템이 정상 작동하는지 확인합니다"

  # 에이전트 상태 확인
  entelecheia-cli agent list

  # 감사 추적 확인
  entelecheia-cli trace-chain demiurge.001
  ```

## 보안 강화 (권장)

| 확인 사항 | 명령어 |
| --- | --- |
| 환경 변수에 시크릿 없음 확인 | `env \| grep -i key` |
| RBAC 그룹 검토 | `entelecheia-cli security rbac-list` |
| 속도 제한 확인 | `entelecheia-cli config show channel.rate_limit` |
| 컨테이너 격리 확인 | `docker inspect entelecheia \| grep SecurityOpt` |
| OreXis 감사 로그 검토 | `entelecheia-cli logs --agent orexis --lines 100` |

## 문제 해결

| 증상 | 진단 |
| --- | --- |
| 에이전트 응답 없음 | `entelecheia-cli status` → scepter 실행 중 확인 |
| LLM 호출 실패 | API 키 확인: `entelecheia-cli config show providers` |
| 컨테이너 오류 | `docker logs entelecheia` → Youki/Docker 오류 확인 |
| 데이터베이스 문제 | `DATABASE_URL` 또는 pglite 데이터 디렉터리 권한 확인 |
| 도구 권한 거부 | `entelecheia-cli security policy-list` → 거부된 호출 검토 |
