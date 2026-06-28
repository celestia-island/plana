# Manejo de Errores — thiserror con Result del crate

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight necesita un tipo de error unificado en todos los módulos (pantalla, SSH, hardware, red, túneles). La biblioteca debe exponer una API de error limpia manteniendo el manejo interno de errores ergonómico.

## Decisión

Usar `thiserror` para derivar el enum `EvernightError` con implementaciones de visualización `#[error(...)]` por variante. Definir `pub type Result<T> = std::result::Result<T, EvernightError>` como el tipo de resultado a nivel de crate. Cada variante captura contexto específico del dominio (ScreenCapture, Ssh, Tunnel, etc.) como `String` en lugar de envolver tipos de error externos, para evitar filtrar detalles de dependencias internas.

## Consecuencias

### Positivas

- API de error pública estable; los consumidores hacen match sobre variantes del enum sin conocer los internos

### Negativas

- Cierta pérdida de información por la conversión a `String`
- Sin encadenamiento estructurado de errores
