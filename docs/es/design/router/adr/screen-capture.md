+++
title = "Arquitectura de Captura de Pantalla"
description = """Registro de decisión arquitectónica — Arquitectura de Captura de Pantalla."""
lang = "es"
category = "design"
subcategory = "router"
+++

# Arquitectura de Captura de Pantalla

- **Estado**: Aceptada
- **Fecha**: 2025-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight requiere un subsistema de captura de pantalla multiplataforma que alimente fotogramas al pipeline de streaming WebRTC. El subsistema debe funcionar en Windows, macOS y Linux con latencia mínima y admitir rutas de captura tanto aceleradas por GPU como basadas en software.

Restricciones clave:

- **Presupuesto de latencia**: la captura a codificación de extremo a extremo debe mantenerse por debajo de 16 ms a 60 FPS
- **Copia cero donde sea posible**: los fotogramas deben llegar al codificador sin copias innecesarias
- **Soporte de conexión en caliente**: pantallas y GPUs pueden aparecer o desaparecer en tiempo de ejecución
- **Manejo de permisos**: macOS requiere permiso de Grabación de Pantalla; Linux requiere acceso al protocolo XDG/X11 o Wayland

## Decisión

Adoptamos un **backend de captura basado en traits** con implementaciones específicas por plataforma seleccionadas en tiempo de compilación mediante funcionalidades de Cargo. Cada backend implementa el trait `FrameProvider`:

```rust
#[async_trait]
pub trait FrameProvider: Send + Sync {
    async fn enumerate_outputs(&self) -> Result<Vec<OutputInfo>>;
    async fn start_capture(&mut self, output: OutputId, config: CaptureConfig) -> Result<FrameReceiver>;
    async fn stop_capture(&mut self, output: OutputId) -> Result<()>;
}
```

### Selección de backend

| Plataforma | Backend principal          | Respaldo            |
|------------|----------------------------|---------------------|
| Windows    | DXGI Desktop Duplication   | GDI BitBlt          |
| macOS      | ScreenCaptureKit (SCStream) | CGWindowListCreateImage |
| Linux      | PipeWire (vía xdg-desktop-portal) | XShm / XFixes |

### Ciclo de vida del fotograma

1. `FrameProvider::start_capture` devuelve un `FrameReceiver` — un canal MPSC acotado que transporta structs `Frame`
2. Cada `Frame` posee un búfer de memoria compartida (`Arc<FrameBuffer>`) que referencia memoria de GPU cuando está disponible
3. El codificador WebRTC consume del canal; cuando todas las referencias `Arc` se liberan, el búfer se devuelve a un pool de reutilización
4. El hilo de captura nunca se bloquea en la codificación — si el canal está lleno, el fotograma más antiguo se descarta y un contador `FrameDropped` se incrementa

### Espacio de color y formato

- Todos los backends negocian el formato de mayor profundidad de bits disponible (BGRA8, NV12 o P010)
- Un paso `ColorSpaceConverter` maneja la transformación al formato preferido del codificador
- Los metadatos HDR se preservan cuando la fuente los proporciona

## Consecuencias

### Positivas

- Separación limpia entre captura y codificación permite pruebas independientes
- La ruta de copia cero en Windows (DXGI) y macOS (ScreenCaptureKit) mantiene la latencia dentro del presupuesto
- El diseño basado en traits permite backends de terceros (p. ej., pantallas virtuales, fuentes de prueba) sin modificar el código central
- El pool de búferes de fotogramas reduce la presión de asignación bajo captura sostenida

### Negativas

- PipeWire en Linux introduce una dependencia de D-Bus que complica escenarios sin entorno gráfico/embebidos
- El permiso de Grabación de Pantalla en macOS requiere interacción del usuario en el primer lanzamiento — no hay alternativa silenciosa
- Mantener cuatro implementaciones de backend aumenta la superficie de pruebas

### Riesgos y mitigaciones

- **Riesgo**: La API de PipeWire cambia entre distribuciones. **Mitigación**: fijarse a la API C estable `pw_stream` y empaquetar los bindings de Rust.
- **Riesgo**: Tartamudeo de DXGI en portátiles con GPU híbrida. **Mitigación**: detectar la topología de GPU al inicio y preferir la GPU integrada para captura cuando la GPU discreta está renderizando.

## Referencias

- [API DXGI Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api)
- [Documentación de ScreenCaptureKit](https://developer.apple.com/documentation/screencapturekit)
- [Portal de captura de pantalla PipeWire](https://docs.flatpak.org/en/latest/portal-api-reference.html#gdbus-org-freedesktop-portal-ScreenCast)
