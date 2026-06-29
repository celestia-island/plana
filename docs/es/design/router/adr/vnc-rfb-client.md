+++
title = "Cliente de Protocolo VNC (RFB)"
description = """Registro de decisión arquitectónica — Cliente de Protocolo VNC (RFB)."""
lang = "es"
category = "design"
subcategory = "router"
+++

# Cliente de Protocolo VNC (RFB)

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight necesita un cliente de escritorio remoto gráfico que admita VNC. El protocolo RFB (RFC 6143) es el estándar para VNC. Opciones: usar un crate Rust existente, vincular a libvncclient o implementar desde cero.

## Decisión

Implementar un cliente RFB 003.008 en Rust puro desde cero. Soporta negociación de versión, negociación de seguridad (None, VncAuth), autenticación de desafío-respuesta DES, negociación de formato de píxeles y codificación Raw/CopyRect. Implementa el trait `ViewportBackend` para integración con frontends.

## Consecuencias

### Positivas

- Sin dependencia de C; control total sobre el flujo del protocolo.

### Negativas

- Las codificaciones ZRLE y Tight aún no están implementadas (son pesadas de implementar).
- Se necesita implementación de DES para autenticación VNC.
