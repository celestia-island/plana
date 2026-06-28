# Desacoplamiento de Módulos y Propiedad de Tipos

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

El código original colocaba todos los tipos de datos en un único monolito `types.rs` — mezclando tipos de pantalla, hardware, remoto, protocolo y túnel. Esto creaba riesgos de dependencia circular y hacía poco claro qué módulo era «dueño» de cada tipo. Agregar una nueva funcionalidad (p. ej., VNC) requería modificar el archivo de tipos compartido.

## Decisión

Mover los tipos de cada dominio a su propio archivo `types.rs` de módulo (p. ej., `screen/types.rs`, `hardware/types.rs`). Los archivos raíz `types.rs` y `prelude.rs` reexportan todos los tipos para compatibilidad hacia atrás. Cada módulo es el único propietario de sus tipos y puede evolucionar de forma independiente.

## Consecuencias

### Positivas

- Propiedad de tipos clara; sin conflictos de tipos entre módulos
- Los nuevos módulos no tocan archivos existentes

### Negativas

- Los consumidores deben importar desde el módulo correcto o usar el prelude
- La capa de reexportación añade una indirección
