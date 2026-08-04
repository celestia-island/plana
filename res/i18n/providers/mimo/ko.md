# Xiaomi MiMo

## 소개

Xiaomi MiMo는 Xiaomi가 개발한 AI 모델 시리즈로, 코딩 및 범용 작업을 위해 설계되었습니다. Token Plan은 여러 리전 클러스터에서 OpenAI 호환 및 Anthropic 호환 API 엔드포인트를 통해 MiMo 모델에 대한 구독 기반 액세스를 제공합니다.

## 조직

Xiaomi Corporation은 MiMo 브랜드로 AI 모델을 개발하는 중국 전자 기업입니다. MiMo 모델은 코딩 지원에 최적화되어 있으며 함수 호출, 스트리밍 및 기타 OpenAI 호환 기능을 지원합니다.

## 토큰 플랜

Token Plan은 구독 기반 액세스 모델로, API 키는 `tp-xxxxx` 형식(종량제 `sk-xxxxx` 키와 다름)을 사용합니다. 사용 가능한 클러스터:

- 중국: `https://token-plan-cn.xiaomimimo.com/v1`
- 싱가포르: `https://token-plan-sgp.xiaomimimo.com/v1`
- 유럽: `https://token-plan-ams.xiaomimimo.com/v1`

## 인증

Token Plan은 사용자 정의 `api-key` 헤더를 사용합니다(표준 `Authorization: Bearer`가 아님). 인증 유형이 `api-key`로 설정되면 시스템이 자동으로 이를 처리합니다.

## 공식 웹사이트

https://platform.xiaomimimo.com
