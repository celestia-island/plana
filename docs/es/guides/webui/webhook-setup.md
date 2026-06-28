+++
title = "Guía de Configuración de Webhooks"
description = """> Audiencia: Administradores que integran servicios externos con shittim-chest."""
lang = "es"
category = "guides"
subcategory = "webui"
+++

# Guía de Configuración de Webhooks

> **Audiencia**: Administradores que integran servicios externos con shittim-chest.
> **Última actualización**: 2026-05-25

## Resumen

Los webhooks permiten que servicios externos (GitHub, GitLab, Gitee) envíen eventos en tiempo real a shittim-chest. Los eventos se validan, parsean y reenvían a scepter, que los despacha al agente apropiado.

```text
Servicio Externo → shittim_chest → scepter → Agente
```

`shittim_chest` también soporta endpoints de webhook personalizados para servicios no soportados nativamente.

## Configuración de Webhook de GitHub

### Paso 1: Configurar el Entorno

Configura el secreto del webhook en tu `.env`:

```bash
WEBHOOK_GITHUB_SECRET=tu-secreto-hmac-aqui
WEBHOOK_PUBLIC_URL=https://tu-dominio.com
```

Genera un secreto fuerte:

```bash
openssl rand -hex 32
```

### Paso 2: Crear el Webhook en GitHub

1. Navega a tu repositorio → **Settings** → **Webhooks** → **Add webhook**
1. Configura **Payload URL** a `https://tu-dominio.com/api/webhook/github`
1. Configura **Content type** a `application/json`
1. Configura **Secret** al mismo valor que `WEBHOOK_GITHUB_SECRET`
1. Selecciona eventos: `push`, `pull_request`, `issues`, `issue_comment`
1. Asegúrate de que **Active** esté marcado
1. Haz clic en **Add webhook**

### Paso 3: Verificar

GitHub enviará un evento `ping` inmediatamente. Revisa la pestaña **Recent Deliveries** para confirmar una respuesta `200`.

## Configuración de Webhook de GitLab

### Paso 1: Configurar el Entorno

```bash
WEBHOOK_GITLAB_SECRET=tu-token-secreto-gitlab
```

### Paso 2: Crear el Webhook en GitLab

1. Navega a tu proyecto → **Settings** → **Webhooks**
1. Configura **URL** a `https://tu-dominio.com/api/webhook/gitlab`
1. Configura **Secret token** al mismo valor que `WEBHOOK_GITLAB_SECRET`
1. Selecciona disparadores: `Push events`, `Merge request events`, `Issue events`
1. Asegúrate de que **Enable SSL verification** esté marcado (para HTTPS)
1. Haz clic en **Add webhook**

### Paso 3: Verificar

Usa el botón **Test** en GitLab para enviar un evento de prueba. Confirma que la entrega tenga éxito.

## Configuración de Webhook de Gitee

Los webhooks de Gitee (码云) también están soportados.

### Paso 1: Configurar el Entorno

Gitee usa el mismo `WEBHOOK_GITLAB_SECRET` para validación HMAC (con token como fallback). Alternativamente, configura `WEBHOOK_GITEE_PASSWORD` si usas autenticación basada en contraseña.

### Paso 2: Crear el Webhook en Gitee

1. Navega a tu repositorio → **Management** → **Webhooks**
1. Configura **URL** a `https://tu-dominio.com/api/webhook/gitee`
1. Configura **Password/Signing Key** al mismo secreto
1. Selecciona eventos: `Push`, `Pull Request`, `Issues`
1. Haz clic en **Add**

## Webhooks Personalizados

`shittim_chest` soporta un endpoint de webhook personalizado genérico en `/api/webhook/custom/{name}`. Para añadir una fuente de webhook personalizada:

1. Configura `WEBHOOK_PUBLIC_URL` en `.env`
1. Configura tu servicio externo para hacer POST a `https://tu-dominio.com/api/webhook/custom/{name}`
1. Los eventos se reenvían a scepter con el nombre del webhook como fuente del evento

Para integrar nuevos proveedores de webhook a nivel de código:

1. Añade un manejador en `packages/core/src/webhook.rs`
1. Implementa validación HMAC o token para el nuevo proveedor
1. Parsea el formato de evento personalizado y reenvía a scepter mediante socket Unix

## Lista Blanca de IPs

`shittim_chest` soporta listas blancas de IPs para fuentes de webhook para rechazar solicitudes de orígenes desconocidos:

```bash
# .env
WEBHOOK_IP_WHITELIST=140.82.112.0/20,192.30.252.0/22  # IPs de GitHub
```

Configura rangos CIDR para cada proveedor de webhook. Las solicitudes desde IPs fuera de la lista blanca son rechazadas.

## Tipos de Eventos

Eventos soportados y su mapeo a disparadores de scepter:

| Fuente | Evento | `event_type` de scepter |
| --- | --- | --- |
| GitHub | `push` | `github.push` |
| GitHub | `pull_request` | `github.pull_request` |
| GitHub | `issues` | `github.issues` |
| GitHub | `issue_comment` | `github.issue_comment` |
| GitLab | `push` | `gitlab.push` |
| GitLab | `merge_request` | `gitlab.merge_request` |
| GitLab | `issues` | `gitlab.issues` |
| Gitee | `push` | `gitee.push` |
| Gitee | `pull_request` | `gitee.pull_request` |
| Gitee | `issues` | `gitee.issues` |

## Registro de Entregas

`shittim_chest` mantiene un registro de entregas de eventos de webhook. Las entregas duplicadas se detectan usando una caché LRU (hasta 10,000 IDs de entrega). Accede a los registros de entrega mediante:

- **API REST**: `GET /api/webhook/deliveries`
- Panel de administración: **Webhooks** → **Registro de Entregas**

## Seguridad

Todos los webhooks deben pasar verificación de firma:

- **GitHub**: Usa la cabecera `X-Hub-Signature-256`. Validado contra `WEBHOOK_GITHUB_SECRET`.
- **GitLab**: Usa la cabecera `X-Gitlab-Token`. Validado contra `WEBHOOK_GITLAB_SECRET`.
- **Gitee**: Usa firma HMAC-SHA256 con fallback de token.

Las solicitudes sin firmas válidas son rechazadas con `401 Unauthorized`. Nunca expongas secretos de webhook en código del lado del cliente o logs.

## Pruebas

Usa el panel de administración para probar la integración de webhooks:

1. Inicia sesión en el panel de administración (por defecto `:3000`)
1. Navega a **Webhooks** en la barra lateral
1. Visualiza los registros de entrega y la configuración
1. Prueba los endpoints mediante la funcionalidad de prueba del servicio externo

También puedes probar manualmente con curl:

```bash
curl -X POST https://tu-dominio.com/api/webhook/github \
  -H "Content-Type: application/json" \
  -H "X-Hub-Signature-256: sha256=<hmac-calculado>" \
  -d '{"action":"push","ref":"refs/heads/main"}'
```

## Solución de Problemas

### 401 Unauthorized

**Causa**: Discordancia de firma HMAC o IP no en la lista blanca.
**Solución**: Asegúrate de que el secreto en `.env` coincida con el secreto configurado en la plataforma de origen. Verifica espacios en blanco al final o problemas de codificación. Verifica la configuración de la lista blanca de IPs.

### 502 Bad Gateway

**Causa**: scepter no es alcanzable.
**Solución**: Verifica `ENTELECHEIA_SCEPTER_URL` y `ENTELECHEIA_TUI_SOCK` en `.env`. Asegúrate de que la instancia de scepter esté en ejecución y la ruta del socket Unix sea accesible.

### Los eventos no llegan a los agentes

**Causa**: Tipo de evento no mapeado o agente no configurado para manejarlo.
**Solución**: Revisa los logs del backend para el `event_type` parseado. Verifica que el agente objetivo tenga un manejador registrado para ese evento. Revisa el registro de entregas mediante API o panel de administración.

### Entregas duplicadas

**Causa**: El servicio externo está reintentando debido a timeout. `shittim_chest` detecta automáticamente duplicados mediante caché LRU.
**Solución**: Si los reintentos válidos están siendo bloqueados, aumenta el tamaño de la caché de IDs de entrega. Asegúrate de que `shittim_chest` responda dentro de la ventana de timeout del servicio (GitHub: 10 segundos).
