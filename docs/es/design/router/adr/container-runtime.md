# Cliente de Tiempo de Ejecución de Contenedores (Docker/Podman)

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

La gestión de contenedores (Docker, Podman) es un caso de uso clave para un gestor de conexiones universal. Las operaciones incluyen listar contenedores, ejecutar shell, ver registros y reenvío de puertos.

## Decisión

Usar la API de Docker Engine mediante socket Unix (o tubería con nombre en Windows). La API compatible con Docker de Podman se admite mediante la misma ruta de código. Definir modelos de contenedor tipados (`ContainerInfo`, `ContainerState`, `ContainerPort`). Futuro: implementar `TerminalBackend` sobre Docker exec attach para shells interactivas de contenedores.

## Consecuencias

### Positivas

- La API de Docker está bien documentada y es estable; la compatibilidad con Podman es gratuita.

### Negativas

- Requiere que el demonio Docker/Podman esté en ejecución.
- Diferencias de versión de API entre versiones de Docker.
