+++
title = "Guía de uso de CLI"
description = """`entelecheia-cli` es la interfaz de línea de comandos de la plataforma de colaboración multi-agente Entelecheia (玄枢). Se comunica con el servidor scepter a través de Unix socket JSON-RPC, proporcionando interacción de chat, gestión del ciclo de vida de servicios, control de agentes, configuración y más."""
lang = "es"
category = "guides"
subcategory = "core"
+++

# Guía de uso de CLI

`entelecheia-cli` es la interfaz de línea de comandos de la plataforma de colaboración multi-agente Entelecheia (玄枢). Se comunica con el servidor scepter a través de Unix socket JSON-RPC, proporcionando interacción de chat, gestión del ciclo de vida de servicios, control de agentes, configuración y más.

> Nota: CLI actualmente no ha alcanzado la misma funcionalidad completa que TUI. Para el estado actual, consulta [ARCHITECTURE.md](../../ARCHITECTURE.md).

---

## Tabla de contenidos

- [Instalación](#instalación)
- [Uso básico](#uso-básico)
- [Opciones globales](#opciones-globales)
- [Comandos de chat](#comandos-de-chat)
- [Gestión de agentes](#gestión-de-agentes)
- [Ciclo de vida de servicios](#ciclo-de-vida-de-servicios)
- [Configuración](#configuración)
- [Contexto de conexión](#contexto-de-conexión)
- [Estado y monitoreo](#estado-y-monitoreo)
- [Suscripción (Layer3)](#suscripción-layer3)
- [Ejecutar agentes](#ejecutar-agentes)
- [Línea de tiempo](#línea-de-tiempo)
- [Imágenes Docker](#imágenes-docker)
- [Uso avanzado](#uso-avanzado)

---

## Instalación

### Construir desde el código fuente

```bash
# Clonar repositorio
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia

# Construir el binario CLI
cargo build --package entelecheia-cli

# O usar just
just cli
```

El binario se encuentra en `target/debug/entelecheia-cli` (debug) o `target/release/entelecheia-cli` (release).

### Binarios preconstruidos

Los binarios preconstruidos están disponibles en [GitHub Releases](https://github.com/celestia-island/entelecheia/releases). Descarga el archivo comprimido adecuado para tu plataforma y coloca el binario en tu `PATH`.

---

## Uso básico

```bash
# Mostrar ayuda
entelecheia-cli --help

# Enviar mensaje a través de la cadena de habilidades
entelecheia-cli send Explica la arquitectura de este proyecto

# Enviar mensaje por tubería
echo "Resume este archivo" | entelecheia-cli send

# Verificar estado del sistema
entelecheia-cli status
```

---

## Opciones globales

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `-l, --log-level <LEVEL>` | Nivel de registro (trace、debug、info、warn、error) | `warn` |
| `-d, --daemon` | Despachar comando en segundo plano y salir inmediatamente | — |
| `-c, --clean` | Limpiar contenedores Cosmos y archivos socket | — |
| `-a, --auto-approve` | Aprobar operaciones automáticamente (asegura que el servidor esté en ejecución) | — |
| `-t, --table` | Salida en tabla legible por humanos (formato ANSI) | Predeterminado |
| `-j, --json` | Salida JSON (legible por máquina) | — |
| `-r, --raw` | Salida de texto plano sin formato | — |
| `--format <FORMAT>` | Formato de salida（table、json、raw） | `table` |

Opciones de formato de salida:

- `table` — Salida en tabla legible por humanos
- `json` — Salida JSON legible por máquina

**Ejemplos:**

```bash
# Limpiar contenedores
entelecheia-cli --clean

# Obtener estado en formato JSON
entelecheia-cli status --format json

# Enviar mensaje en modo depuración
entelecheia-cli -l debug send "Depurar problema de conexión"

# Ejecutar agente en modo segundo plano (retorna inmediatamente)
entelecheia-cli -d run my-agent --ci
```

---

## Comandos de chat

El subcomando `chat` gestiona la interacción de diálogo con el sistema de agentes de sesión.

### Enviar mensaje

```bash
entelecheia-cli chat send [OPTIONS]
```

| Opción | Descripción |
| --- | --- |
| `-m, --message <MSG>` | Texto del mensaje a enviar |
| `--stdin` | Leer mensaje desde la entrada estándar |
| `-f, --file <PATH>` | Leer mensaje desde un archivo |

Solo se puede usar una fuente de entrada a la vez.

**Ejemplos:**

```bash
# Enviar mensaje directamente
entelecheia-cli chat send -m "Hola, ¿qué puedes hacer?"

# Desde entrada estándar
echo "Analiza el código en src/main.rs" | entelecheia-cli chat send --stdin

# Desde archivo
entelecheia-cli chat send -f ./prompts/review.txt
```

El comando `chat send` envía el mensaje a través de la **cadena de habilidades** — el pipeline de ejecución central que coordina múltiples agentes. El progreso se muestra durante la ejecución mediante una animación giratoria.

### Historial de chat

```bash
entelecheia-cli chat history [OPTIONS]
```

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--conversation <ID>` | Filtrar por ID de conversación | — |
| `--agent <TYPE>` | Filtrar por tipo de agente | — |
| `--role <ROLE>` | Filtrar por rol（user/assistant/system） | — |
| `--from <ISO8601>` | Fecha/hora de inicio（ISO 8601） | — |
| `--to <ISO8601>` | Fecha/hora de fin（ISO 8601） | — |
| `--limit <N>` | Número máximo de mensajes a devolver | `50` |
| `--offset <N>` | Desplazamiento de paginación | `0` |

**Ejemplo:**

```bash
entelecheia-cli chat history --agent ApoRia --limit 20 --from 2026-05-01T00:00:00Z
```

### Mensajes recientes

```bash
entelecheia-cli chat recent [OPTIONS]
```

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--timeline <ID>` | Filtrar por ID de línea de tiempo / sesión | — |
| `--agent <TYPE>` | Filtrar por tipo de agente | — |
| `--limit <N>` | Número máximo de mensajes a devolver | `20` |

---

## Gestión de agentes

Gestiona el ciclo de vida de los agentes (listar, iniciar, detener, reiniciar).

```bash
entelecheia-cli agent <COMMAND>
```

### Comandos

```bash
# Listar todos los agentes y su estado
entelecheia-cli agent list

# Iniciar agente por tipo
entelecheia-cli agent start <AGENT_TYPE>

# Detener agente en ejecución
entelecheia-cli agent stop <AGENT_TYPE>

# Reiniciar agente
entelecheia-cli agent restart <AGENT_TYPE>
```

**Tipos de agente disponibles:** ApoRia、EleOs、EpieiKeia、Haplotes、HubRis、Kalos、NeiKos、OreXis、PhiLia、Polemos、SkeMma、SkoPeo。

> Nota: Los agentes se ejecutan como crates de biblioteca dentro del tiempo de ejecución de scepter, no como ejecutables independientes. El comando `agent start` intenta generar un binario que coincida con el nombre del agente, lo cual se aplica principalmente cuando los agentes se compilan como binarios separados. En el uso real, los agentes se activan a través del servidor scepter.

---

## Ciclo de vida de servicios

Gestiona la pila de servicios de Entelecheia (玄枢) usando contenedores Docker.

### Inicializar servicios

```bash
entelecheia-cli init [OPTIONS]
```

Configura la pila de servicios completa: PostgreSQL (con pgvector), registro Docker, servidor scepter y WebUI. Crea la red Docker requerida y extrae/construye imágenes.

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--prefix <STR>` | Prefijo de nombre de contenedor | `e-` |
| `--source-build` | Construir imágenes desde código fuente en lugar de extraer | `false` |
| `--webui-port <PORT>` | Puerto de WebUI | `3424` |

**Ejemplo:**

```bash
entelecheia-cli init --prefix ent- --webui-port 8080
```

### Iniciar todos los servicios

```bash
entelecheia-cli serve [OPTIONS]
```

Inicia todos los contenedores previamente inicializados. Requiere ejecutar `init` primero.

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--prefix <STR>` | Prefijo de nombre de contenedor | `e-` |
| `--webui-port <PORT>` | Puerto de WebUI | `3424` |

### Detener todos los servicios

```bash
entelecheia-cli stop [OPTIONS]
```

Detiene todos los contenedores en ejecución en orden: webui → scepter → registry → postgres.

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--prefix <STR>` | Prefijo de nombre de contenedor | `e-` |

### Iniciar solo WebUI

```bash
entelecheia-cli webui [OPTIONS]
```

Inicia o crea solo el contenedor WebUI.

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--prefix <STR>` | Prefijo de nombre de contenedor | `e-` |
| `--webui-port <PORT>` | Puerto de WebUI | `3424` |

---

## Configuración

Ver y validar la configuración del sistema.

### Mostrar configuración

```bash
entelecheia-cli config show
```

Muestra la configuración actual, incluyendo:

- URL de base de datos y configuración de conexión
- Configuración del proveedor LLM de ApoRia (nombre, modelo, endpoint)
- Dirección de enlace WebSocket
- Nivel de registro

Las claves API se ocultan en la salida (mostradas como `***`).

### Validar configuración

```bash
entelecheia-cli config validate
```

Ejecuta comprobaciones de validación:

- URL de base de datos configurada
- Al menos un proveedor ApoRia configurado con ajustes completos
- Dirección de enlace WebSocket configurada

Devuelve resultado aprobado/fallido con detalles de cualquier problema.

**Ejemplo de salida:**

```text
Validate Configuration:

Validating database configuration...
  [ OK ]  Database URL set

Validating ApoRia LLM configuration...
  [ OK ]  ApoRia providers configured

Validating WebSocket configuration...
  [ OK ]  WebSocket Bind Address set

[ OK ]  Configuration validation passed
```

---

## Contexto de conexión

El subcomando `context` se usa para gestionar perfiles de conexión con nombre, permitiéndote cambiar entre servidores scepter locales (Unix socket) y remotos (WebSocket). Su uso es similar al comando `docker context` de Docker.

### Concepto

Un **contexto** es un perfil de configuración con nombre que registra cómo CLI se conecta al servidor scepter:

- **local** — Conexión Unix socket (predeterminado, se resuelve automáticamente a `/run/.../entelecheia-tui.sock`)
- **remote** — Conexión WebSocket con autenticación Bearer token

Los contextos se almacenan en `~/.config/entelecheia/contexts/contexts.toml`.

### Listar contextos

```bash
entelecheia-cli context list
```

El contexto actualmente activo se marca con `*`.

### Mostrar contexto actual

```bash
entelecheia-cli context show
```

Muestra el tipo, ruta de socket, URL WS e información de descripción del contexto activo.

### Crear contexto

```bash
# Contexto WebSocket remoto
entelecheia-cli context create staging \
  --ws-url ws://scepter.example.com:8424/ws \
  --bearer-token <TOKEN> \
  --description "Servidor de staging"

# Contexto local adicional
entelecheia-cli context create dev --description "Servidor de desarrollo"
```

Obtener Bearer token del servidor remoto:

```bash
# En la máquina del servidor
docker exec e-scepter cat /home/entelecheia/.config/entelecheia/scepter.token
```

### Cambiar contexto

```bash
entelecheia-cli context use staging
# A partir de ahora, todos los comandos (send、status、chat, etc.) se enrutarán a través de la conexión de staging
```

### Eliminar contexto

```bash
entelecheia-cli context remove staging
```

El contexto `default` no se puede eliminar.

### Flujo de trabajo de ejemplo

```bash
# Ver contexto actual
entelecheia-cli context list

# Crear contexto remoto para servidor de pre-lanzamiento
entelecheia-cli context create staging \
  --ws-url ws://192.168.1.100:8424/ws \
  --bearer-token $(cat /path/to/token)

# Cambiar al entorno de pre-lanzamiento
entelecheia-cli context use staging

# Enviar mensaje a través del servidor remoto
entelecheia-cli send "Listar tareas pendientes actuales"

# Verificar estado del servidor remoto
entelecheia-cli status

# Volver al local
entelecheia-cli context use default
```

---

## Estado y monitoreo

### Estado del sistema

```bash
entelecheia-cli status
```

Muestra:

- Versión del servidor
- Estado de conexión (estado del socket)
- Resumen del proveedor LLM
- Dirección de enlace WebSocket
- Lista de agentes y estado de ejecución/detención
- Recursos del sistema (uso de memoria, carga promedio)

### Consulta de ruta de estado

El comando `status` acepta parámetros tipo ruta para consultar subsistemas específicos. La sintaxis admite líneas de tiempo con ámbito de agente, verificación de historial de chat y enumeración de dispositivos.

```bash
entelecheia-cli status <PATH> [--raw]
```

| Sintaxis de ruta | Descripción |
| --- | --- |
| `timeline.#agent[-N]` | Muestra el historial de las últimas N invocaciones de skill de un agente |
| `timeline.#agent[N][M]` | Muestra la M-ésima llamada MCP/herramienta en la N-ésima invocación de skill |
| `history[-N]` | Muestra los últimos N mensajes de chat (todos los roles) |
| `history[-N].body` | Muestra el cuerpo del N-ésimo mensaje desde el final |
| `device` | Lista todos los dispositivos periféricos reconocidos por Polemos |
| `device[N]` | Muestra detalles del N-ésimo dispositivo Polemos |

**Ejemplos:**

```bash
# Historial de las últimas 30 invocaciones de skill del agente Haplotes #001
entelecheia-cli status timeline.#hap_lotes.001[-30]

# La 2ª llamada MCP/herramienta de la 3ª invocación de skill
entelecheia-cli status timeline.#hap_lotes.001[3][2]

# Últimos 30 mensajes
entelecheia-cli status history[-30]

# Cuerpo del 3er mensaje desde el final (texto plano)
entelecheia-cli status history[-3].body --raw

# Todos los dispositivos Polemos
entelecheia-cli status device

# Detalles del 3er dispositivo Polemos
entelecheia-cli status device[3]
```

> **Nota para Shell:** En bash/zsh, envuelve las rutas que contienen `[...]` con comillas simples para evitar la expansión glob: `entelecheia-cli status 'history[-30]'`. El carácter `#` incrustado en medio de una palabra no necesita escape. En fish shell, ninguna de las rutas anteriores necesita comillas.

La consulta de ruta de estado se comunica con el servidor a través de Unix socket JSON-RPC. Las consultas `timeline.*` e `history.*` requieren que el servidor esté en ejecución. Las consultas `device` requieren que el espacio de trabajo Polemos esté registrado en el servidor.

### Ver registros

```bash
entelecheia-cli logs [OPTIONS]
```

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `-a, --agent <NAME>` | Filtrar registros por nombre de agente | Todos los agentes |
| `-l, --lines <N>` | Número de líneas a mostrar (cola) | `100` |

**Ejemplos:**

```bash
# Mostrar las últimas 200 líneas de todos los registros de agentes
entelecheia-cli logs -l 200

# Mostrar registros de ApoRia
entelecheia-cli logs -a ApoRia
```

Los registros se leen del directorio `./logs/`. Cada agente tiene su propio archivo de registro (`ApoRia.log`、`EleOs.log`, etc.).

---

## Suscripción (Layer3)

Gestiona suscripciones de agentes Layer3 — paquetes de agentes externos que se pueden instalar y ejecutar.

### Listar suscripciones

```bash
entelecheia-cli subscribe list
```

Muestra todas las suscripciones configuradas, incluyendo estado (instalado/pendiente), habilitación, configuración de actualización automática y origen.

### Añadir suscripción

```bash
entelecheia-cli subscribe add [OPTIONS]
```

| Opción | Descripción |
| --- | --- |
| `--name <NAME>` | Nombre de la suscripción (obligatorio) |
| `--source <SOURCE>` | Tipo de origen：`official`、`github` o `url` (obligatorio) |
| `--repository <REPO>` | Repositorio GitHub (para origen github) |
| `--url <URL>` | URL directa (para origen url) |
| `--version <VER>` | Restricción de versión |
| `--auto-update` | Habilitar actualización automática |
| `--disabled` | Añadir como deshabilitada |

**Ejemplo:**

```bash
entelecheia-cli subscribe add --name my-agent --source github --repository user/repo
```

### Eliminar suscripción

```bash
entelecheia-cli subscribe remove <NAME>
```

### Sincronizar suscripciones

```bash
# Sincronizar todas las suscripciones
entelecheia-cli subscribe sync

# Sincronizar una suscripción específica
entelecheia-cli subscribe sync --name my-agent
```

### Actualización automática

```bash
entelecheia-cli subscribe auto-update
```

Actualiza todas las suscripciones que tienen `auto_update` habilitado.

---

## Ejecutar agentes

```bash
entelecheia-cli run <AGENT> [OPTIONS]
```

Ejecuta scripts de agentes Layer3. Busca `.amphoreus/<AGENT>/run.py` en el directorio actual. En la primera ejecución se realiza una auditoría de pre-verificación.

| Opción | Descripción |
| --- | --- |
| `--ci` | Habilitar modo CI |
| `--auto-pr` | Habilitar modo PR automático |
| `--dry-run` | Simulación (sin cambios reales) |
| `--providers <LIST>` | Lista de proveedores separada por comas |
| `--output-dir <DIR>` | Directorio de salida |

**Ejemplos:**

```bash
# Ejecutar agente Layer3 en modo simulación
entelecheia-cli run my-agent --dry-run

# Ejecutar con proveedores especificados
entelecheia-cli run my-agent --providers openai,anthropic

# Modo CI con PR automático
entelecheia-cli run my-agent --ci --auto-pr

# Ejecutar en modo segundo plano (retorna inmediatamente, el proceso hijo se ejecuta en segundo plano)
entelecheia-cli -d run my-agent --ci --auto-pr
```

### Modo segundo plano (`-d` / `--daemon`)

El indicador de modo segundo plano hace que CLI regenere un proceso hijo separado eliminando el parámetro `--daemon` y retorne inmediatamente. El proceso hijo hereda el comando original y se ejecuta de forma independiente. Después se puede usar `status` para ver el progreso.

Adecuado para operaciones de larga duración como `run`、`init`、`deploy`:

```bash
# Despachar ejecución de agente en segundo plano
entelecheia-cli -d run my-agent

# Despachar inicialización de servicio en segundo plano
entelecheia-cli -d init --prefix prod-

# Ver estado más tarde
entelecheia-cli status
entelecheia-cli status history[-5]
```

---

## Línea de tiempo

Ver líneas de tiempo de sesión.

### Listar líneas de tiempo

```bash
entelecheia-cli timeline list [OPTIONS]
```

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--agent <TYPE>` | Filtrar por tipo de agente | — |
| `--limit <N>` | Número máximo de resultados | `50` |
| `--offset <N>` | Desplazamiento de paginación | `0` |

### Mostrar detalles de línea de tiempo

```bash
entelecheia-cli timeline show <CONVERSATION_ID> [OPTIONS]
```

| Opción | Descripción | Valor predeterminado |
| --- | --- | --- |
| `--include-messages` | Incluir mensajes en la salida | `true` |

---

## Imágenes Docker

```bash
entelecheia-cli init-docker-images [OPTIONS]
```

Construye o extrae las imágenes Docker requeridas por la plataforma.

| Opción | Descripción |
| --- | --- |
| `--source-build` | Construir imágenes desde código fuente en lugar de extraer |
| `--tag <TAG>` | Etiqueta de imagen (predeterminado: `latest`) |

**Ejemplos:**

```bash
# Construir todas las imágenes desde código fuente
entelecheia-cli init-docker-images --source-build

# Extraer con etiqueta personalizada
entelecheia-cli init-docker-images --tag v0.2.0
```

Imágenes gestionadas:

- `entelecheia` — Servidor de orquestación (con tiempo de ejecución cosmos integrado)
- `pgvector/pgvector` — PostgreSQL con extensión de vectores

---

## Uso avanzado

### Salida JSON para scripting

Usa `--format json` para obtener salida legible por máquina, que se puede canalizar a `jq` u otras herramientas:

```bash
entelecheia-cli status --format json | jq '.server_version'
entelecheia-cli chat history --format json | jq '.messages[].content'
```

### Limpieza e inicialización encadenadas

```bash
# Desmontaje completo y reconstrucción
entelecheia-cli --clean && entelecheia-cli init --prefix my-
```

### Modo depuración

```bash
# Habilitar registro de nivel trace para depuración
entelecheia-cli -l trace send "Mensaje de prueba"
```

### Uso junto con TUI

CLI y TUI se conectan al mismo servidor scepter. Ambos se pueden usar simultáneamente:

- Inicia TUI para sesiones interactivas: `cargo run --bin entelecheia-tui`
- Usa CLI para scripting, automatización y consultas rápidas

---

## Solución de problemas

### "No command specified"

Ejecuta `--help` para ver los comandos disponibles, o usa `send "mensaje"` para enviar un mensaje rápidamente.

### "Failed to connect to Docker"

Asegúrate de que Docker (o Podman) esté en ejecución:

```bash
docker info
docker run hello-world
```

### "Agent binary not found"

Los agentes son crates de biblioteca internos del tiempo de ejecución de scepter, no binarios independientes. Inicia el servidor scepter para activar los agentes:

```bash
entelecheia-cli init && entelecheia-cli serve
```

### "No LLM providers configured"

Configura los proveedores ApoRia a través de variables de entorno. Consulta la [Guía de construcción](building.md) para instrucciones de configuración de proveedores.

### "Configuration validation failed"

Ejecuta `entelecheia-cli config validate` para ver qué comprobaciones fallaron. Problemas comunes:

- Falta la variable de entorno `DATABASE_URL`
- Configuración incompleta del proveedor ApoRia (nombre, modelo, `api_key`)
- Falta la dirección de enlace WebSocket
