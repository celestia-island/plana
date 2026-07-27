# Xiaomi MiMo

## Introducción

Xiaomi MiMo es una serie de modelos de IA desarrollados por Xiaomi, diseñados para tareas de programación y propósito general. El Token Plan ofrece acceso mediante suscripción a los modelos MiMo con endpoints API compatibles con OpenAI y Anthropic en múltiples clústeres regionales.

## Organización

Xiaomi Corporation es una empresa china de electrónica que desarrolla modelos de IA bajo la marca MiMo. Los modelos MiMo están optimizados para la asistencia en programación y soportan llamadas a funciones, streaming y otras características compatibles con OpenAI.

## Plan de Tokens

El Token Plan es un modelo de acceso basado en suscripción donde las claves API utilizan el formato `tp-xxxxx` (distinto de las claves `sk-xxxxx` de pago por uso). Clústeres disponibles:

- China: `https://token-plan-cn.xiaomimimo.com/v1`
- Singapur: `https://token-plan-sgp.xiaomimimo.com/v1`
- Europa: `https://token-plan-ams.xiaomimimo.com/v1`

## Autenticación

El Token Plan utiliza una cabecera personalizada `api-key` (no el estándar `Authorization: Bearer`). El sistema lo maneja automáticamente cuando el tipo de autenticación se configura como `api-key`.

## Sitio Web Oficial

https://platform.xiaomimimo.com
