+++
title = "Estrategia de Frontend Incrustado"
description = """shittim-chest soporta dos modos de alojamiento del frontend: en modo Dev, `dev.py` vigila las fuentes del frontend y dispara `pnpm build` ante cambios, con el backend sirviendo tanto archivos estáticos como API en `:3000`; e"""
lang = "es"
category = "design"
subcategory = "webui"
+++

# Estrategia de Frontend Incrustado

## Resumen

shittim-chest soporta dos modos de alojamiento del frontend: en modo Dev, `dev.py` vigila las fuentes del frontend y dispara `pnpm build` ante cambios, con el backend sirviendo tanto archivos estáticos como API en `:3000`; en modo Release, los archivos estáticos del frontend se incrustan en el binario Rust en tiempo de compilación y se sirven en `:80`. Los modos se alternan mediante la feature de Cargo `embedded-frontend`, con compilación condicional a nivel de código usando `#[cfg(feature = "embedded-frontend")]`.

## Comparación de Arquitectura

```mermaid
flowchart TB
    subgraph Dev[Modo Dev: dev.py + Backend]
        D1[dev.py vigila src del frontend] --> D2[pnpm build → dist/]
        D2 --> D3[shittim_chest :3000 sirve estáticos + API]
    end
    subgraph Release[Modo Release: Incrustado]
        R1[Navegador] --> R2[shittim_chest :80]
        R2 --> R3[API + LLM]
        R2 --> R4[/static/*\nSPA Incrustada]
    end
```

| Dimensión | Dev (sin feature) | Release (embedded-frontend) |
| --- | --- | --- |
| Fuente del frontend | Construido por Vite, servido por el backend | `include_dir!` incrustación en tiempo de compilación |
| Hot reload | Reconstrucción automática mediante dev.py | No soportado (estático) |
| Enrutamiento de solicitudes API | Conexión directa del navegador (mismo origen) | Conexión directa del navegador |
| Tamaño del binario | Solo backend | + directorio dist/ del frontend |
| Requiere Node | Sí (solo build) | No |
| Método de inicio | `dev.py` (vigila + reconstruye) | `just up` lanzamiento único |

## Detalles de Implementación

### Compilación Condicional

```rust
# [cfg(feature = "embedded-frontend")]
static ARONA_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../dist/arona");

async fn serve_arona() -> impl IntoResponse {
    #[cfg(feature = "embedded-frontend")]
    {
        // Leer desde Dir incrustado en tiempo de compilación
    }
    #[cfg(not(feature = "embedded-frontend"))]
    {
        // Leer desde el sistema de archivos ./dist/arona/index.html
    }
}
```

La compilación condicional opera a **nivel de cuerpo de función** en lugar de a nivel de módulo, manteniendo la API pública idéntica en ambos modos.

### Fallback SPA

La aplicación es una single-page application. Todas las rutas que no coinciden con activos estáticos devuelven `index.html`:

```text
GET /               → index.html
GET /chat/123       → index.html (el enrutador del frontend maneja)
GET /backend        → index.html
GET /backend/providers → index.html (el enrutador del frontend maneja)
```

### Detección de Tipo MIME

El servidor de archivos estáticos devuelve el Content-Type correcto basado en la extensión del archivo:

| Extensión | Content-Type |
| --- | --- |
| `.js` | `application/javascript` |
| `.css` | `text/css` |
| `.html` | `text/html` |
| `.json` | `application/json` |
| `.png` | `image/png` |
| `.svg` | `image/svg+xml` |
| `.woff/.woff2` | `font/woff2` |
| Otro | `application/octet-stream` |

## Build del Frontend en el Dockerfile

```text
Etapa 1 (frontend):
  node:22-slim → pnpm install → pnpm build:all → /app/dist/arona/

Etapa 2 (builder):
  rust:1.85-slim → COPY /app/dist/ → cargo build --features embedded-frontend

Etapa 3 (runtime):
  debian:bookworm-slim → COPY binary → ENTRYPOINT ["./shittim_chest"]
```

El build del frontend y la compilación Rust se completan dentro del mismo Dockerfile. La imagen de runtime final contiene solo el binario compilado.

## Decisiones de Diseño

1. **El modo Dev usa dev.py para reconstrucción automática**: `dev.py` vigila las fuentes del frontend y reconstruye ante cambios, con el backend sirviendo todo en un solo puerto.
1. **El modo Release no requiere un proxy inverso**: El binario incrusta la SPA, permitiendo despliegue en un solo proceso y reduciendo la complejidad operativa.
1. **El frontend no se carga dinámicamente en tiempo de ejecución**: Evita dependencias del sistema de archivos e inconsistencia de versiones. La imagen Release contiene solo un único archivo binario.
1. **SPA única**: El frontend se sirve en `/` con el panel de administración en `/backend`.
