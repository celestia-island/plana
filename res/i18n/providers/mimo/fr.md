# Xiaomi MiMo

## Introduction

Xiaomi MiMo est une série de modèles d'IA développés par Xiaomi, conçus pour des tâches de programmation et d'usage général. Le Token Plan offre un accès par abonnement aux modèles MiMo avec des points de terminaison API compatibles OpenAI et Anthropic sur plusieurs clusters régionaux.

## Organisation

Xiaomi Corporation est une entreprise électronique chinoise qui développe des modèles d'IA sous la marque MiMo. Les modèles MiMo sont optimisés pour l'assistance à la programmation et prennent en charge les appels de fonction, le streaming et d'autres fonctionnalités compatibles OpenAI.

## Plan de Jetons

Le Token Plan est un modèle d'accès par abonnement où les clés API utilisent le format `tp-xxxxx` (différent des clés `sk-xxxxx` en paiement à l'usage). Clusters disponibles :

- Chine : `https://token-plan-cn.xiaomimimo.com/v1`
- Singapour : `https://token-plan-sgp.xiaomimimo.com/v1`
- Europe : `https://token-plan-ams.xiaomimimo.com/v1`

## Authentification

Le Token Plan utilise un en-tête personnalisé `api-key` (et non le standard `Authorization: Bearer`). Le système gère cela automatiquement lorsque le type d'authentification est défini sur `api-key`.

## Site Officiel

https://platform.xiaomimimo.com
