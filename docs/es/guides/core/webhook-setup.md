+++
title = "Configuración de plataforma Webhook"
description = """> Descripción del diseño actual de webhook y el alcance de integración"""
lang = "es"
category = "guides"
subcategory = "core"
+++

# Configuración de plataforma Webhook

> Descripción del diseño actual de webhook y el alcance de integración

## Descripción general

El repositorio ya contiene integraciones de webhook orientadas a plataformas de alojamiento de código y plataformas de chat, pero el conjunto aún se encuentra en una etapa de transición y no es una solución completamente unificada y madura.

La estructura de directorios actual contiene simultáneamente:

- Antiguos directorios por plataforma, como `plugins/github-webhook/github/`、`gitee/`、`gitlab/`、`telegram/`、`qq/`、`lark/`
- Una implementación TypeScript más reciente: `plugins/github-webhook/ts/`

El paquete TypeScript actualmente integra:

- GitHub
- Gitee
- GitLab
- Feishu / Lark
- QQ
- Discord
- Telegram

## Qué se puede hacer actualmente

- Recibir eventos de webhook o bot
- Reenviar eventos a Scepter mediante WebSocket o llamadas auxiliares HTTP
- Proporcionar interfaz de verificación de salud `/health` en el servicio TypeScript

## Qué no se puede garantizar por defecto actualmente

- Que todas las plataformas tengan un esquema de despliegue unificado y estable
- Que cada plataforma haya formado una cadena de skills completa impulsada por issues
- Que todas las integraciones de plataforma hayan alcanzado la misma madurez

## Paquete TypeScript

Ubicación: `plugins/github-webhook/ts/`

Modo de ejecución en desarrollo:

```bash
cd plugins/github-webhook/ts
npm install
npm run dev
```

Modo de construcción para producción:

```bash
cd plugins/github-webhook/ts
npm run build
npm start
```

## Variables de entorno clave

- `PORT`: puerto del servicio webhook, predeterminado `8000`
- `SCEPTER_URL`: dirección de reenvío HTTP, predeterminado `http://localhost:8424`
- `SCEPTER_WS_URL`: dirección de reenvío WebSocket, predeterminado `ws://localhost:8424/ws`

## Recomendaciones de uso

Las capacidades de webhook pueden considerarse como "ya existentes, pero con madurez desigual". Si dependes de una plataforma en particular, verifica primero la implementación real del router o bot correspondiente en `plugins/github-webhook/` antes de decidir describirla como utilizable de forma estable en producción.
