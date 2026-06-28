+++
title = "Convenciones de Logging del CLI"
description = """La salida de logs del CLI wrapper de shittim-chest sigue convenciones consistentes con entelecheia, usando el ecosistema `tracing`, emitiendo a stderr en un formato compacto legible por humanos."""
lang = "es"
category = "design"
subcategory = "webui"
+++

# Convenciones de Logging del CLI

## Resumen

La salida de logs del CLI wrapper de shittim-chest sigue convenciones consistentes con entelecheia, usando el ecosistema `tracing`, emitiendo a stderr en un formato compacto legible por humanos.

## Selección del Framework

| Componente | Elección | Razón |
| --- | --- | --- |
| Framework de logging | `tracing` | Estándar del ecosistema Rust, consistente con entelecheia |
| Subscriber | `tracing-subscriber` capa fmt | Salida compacta, sin necesidad de parseo JSON |
| Formato de tiempo | `ShortTimer` (HH:MM:SS) | Amigable para terminal, consistente con el CLI de entelecheia |
| Destino de salida | stderr | Separado de stdout, no interfiere con pipes |

## Código de Inicialización

```rust
use chrono::Local;
use tracing_subscriber::fmt::time::FormatTime;

struct ShortTimer;

impl FormatTime for ShortTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = Local::now();
        write!(w, "{} ", now.format("%H:%M:%S"))
    }
}

// Inicialización
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::new(&args.log_level))
    .with_target(false)          // ocultar rutas de módulos
    .with_timer(ShortTimer)      // formato HH:MM:SS
    .compact()                   // modo compacto
    .with_writer(std::io::stderr) // salida a stderr
    .init();
```

## Comparación de Formatos

| Modo | Ejemplo de Salida | Caso de Uso |
| --- | --- | --- |
| CLI (actual) | `14:23:05  INFO creando red shittim-chest...` | Desarrollo, operaciones |
| Servidor (futuro) | `{"timestamp":"...","level":"INFO","message":"..."}` | Recolección de logs en producción |

## Parámetro --log-level

El CLI acepta el parámetro `--log-level` / `-l` (por defecto `info`):

```text
shittim-chest --log-level debug dev
shittim-chest -l trace status
```

Niveles soportados: `trace`, `debug`, `info`, `warn`, `error`.

## Convenciones de Uso de Niveles de Log

| Nivel | Propósito | Escenarios Típicos del CLI |
| --- | --- | --- |
| `info` | Operaciones importantes | Crear/iniciar/detener contenedor, inicio/fin de migración |
| `warn` | Problemas potenciales | Reintentos de migración, contenedor existe pero en estado anormal |
| `error` | Errores | Fallo del contenedor, fallo de migración, fallo de creación de red |
| `debug` | Información de depuración | (Actualmente no usado, reservado para futuro) |
| `trace` | Flujo detallado | (Actualmente no usado, reservado para futuro) |

## Principios de Diseño

1. **El CLI no traga errores**: Todos los errores se propagan hacia arriba mediante `anyhow::Result`; `main()` imprime automáticamente la cadena de error.
1. **Cada inicio de operación tiene un log**: `creando red...`, `ejecutando migraciones...`, `construyendo shittim_chest...` — el usuario sabe lo que el CLI está haciendo.
1. **Cada finalización de operación tiene confirmación**: `shittim-chest iniciado en 0.0.0.0:80`, `todos los servicios iniciados`.
1. **Las operaciones que tienen éxito silenciosamente no se registran**: `ensure_network` no imprime si la red ya existe, para evitar ruido.
1. **Los logs de contenedores se obtienen mediante la API Docker**: El CLI en sí no escribe logs de negocio, solo logs de operaciones de orquestación.

## Alineación con entelecheia

| Característica | CLI de entelecheia | CLI de shittim-chest | Alineado |
| --- | --- | --- | --- |
| Framework | `tracing` | `tracing` | ✅ |
| Formato de tiempo | `ShortTimer` (HH:MM:SS) | `ShortTimer` (HH:MM:SS) | ✅ |
| Destino de salida | stderr | stderr | ✅ |
| Modo compacto | `.compact()` | `.compact()` | ✅ |
| Ocultar target | `.with_target(false)` | `.with_target(false)` | ✅ |
| --log-level | Soportado | Soportado | ✅ |

La salida de logs del CLI de ambos proyectos es visualmente idéntica, facilitando a los desarrolladores cambiar entre los dos proyectos.
