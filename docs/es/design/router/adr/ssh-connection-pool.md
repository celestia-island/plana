# Pool de Conexiones SSH

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

El código original abría una nueva conexión SSH para cada operación (listar archivos, ejecutar comando). Esto es costoso (TCP + intercambio de claves + autenticación por llamada) y no escala para uso interactivo con múltiples operaciones concurrentes.

## Decisión

Implementar `SshConnectionPool` indexado por `(host, puerto, nombre de usuario)`. Las conexiones se establecen de forma diferida en la primera solicitud y se reutilizan entre operaciones. `PooledSshClient` envuelve una `Arc<Mutex<SshSession>>` compartida. Las verificaciones de salud periódicas eliminan conexiones muertas.

## Consecuencias

### Positivas

- Reducción drástica de latencia para operaciones repetidas.
- Permite sesiones concurrentes de shell + archivos + terminal sobre una única conexión.

### Negativas

- El pool debe manejar la expiración y reconexión de conexiones.
- Las conexiones de larga duración pueden ser terminadas por firewalls.
