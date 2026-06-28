# Arquitectura de Feature Flags

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

El grafo de dependencias monolítico de Evernight obliga a todos los consumidores a incluir cada dependencia (`webrtc`, `russh`, `screenshots`, `sysinfo`) incluso si solo necesitan SSH o telemetría de hardware. Esto aumenta los tiempos de compilación y los tamaños de binario para los usuarios descendentes.

## Decisión

Usar feature flags de Cargo para dividir el crate en las siguientes funcionalidades:

| Funcionalidad | Controla                                           |
|---------------|----------------------------------------------------|
| `screen`      | Módulo de captura de pantalla + crate `screenshots` |
| `webrtc`      | Módulo WebRTC + crate `webrtc` (implica `screen`) |
| `remote-ssh`  | Módulo manejador SSH + crate `russh`               |
| `hardware`    | Módulo de telemetría de hardware + crate `sysinfo` |
| `protocol`    | Tipos de protocolo/mensajes (sin dependencias pesadas) |
| `tunnel`      | Módulo de túnel TCP                                |
| `full`        | Todas las funcionalidades (por defecto)             |

Cada funcionalidad controla tanto el módulo como sus dependencias. La funcionalidad `webrtc` implica `screen` ya que las sesiones WebRTC requieren captura de pantalla.

## Consecuencias

### Positivas

- Los consumidores solo compilan lo que necesitan
- Tiempo de compilación reducido para uso parcial

### Negativas

- La matriz de feature flags crece; se debe probar cada combinación de funcionalidades en CI
