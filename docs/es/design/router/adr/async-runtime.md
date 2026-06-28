# Tiempo de ejecución asíncrono — tokio

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight necesita un tiempo de ejecución asíncrono para E/S de red (SSH, WebRTC, túneles TCP), operaciones basadas en temporizadores (bucles de captura de fotogramas, tiempos de espera de protocolos) y gestión concurrente de tareas. El ecosistema Rust ofrece tokio, async-std y smol.

## Decisión

Usar `tokio` como el único tiempo de ejecución asíncrono. tokio es el estándar de facto en el ecosistema asíncrono de Rust con el soporte de controladores más amplio. Las dependencias clave (`russh`, `webrtc`, `reqwest`) ya requieren tokio. Usar `tokio::runtime::Handle::current()` para generar tareas desde contextos síncronos.

## Consecuencias

### Positivas

- Compatibilidad con el ecosistema; no se necesita puente entre tiempos de ejecución

### Negativas

- tokio es una dependencia pesada
- No se pueden usar crates de async-std o smol que carezcan de soporte para tokio
