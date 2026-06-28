+++
title = "Guía de construcción"
description = """- [Requisitos previos](#requisitos-previos)"""
lang = "es"
category = "guides"
subcategory = "core"
+++

# Guía de construcción

---

## Tabla de contenidos

- [Requisitos previos](#requisitos-previos)
- [Instalación](#instalación)
- [Configuración](#configuración)
- [Construcción](#construcción)
- [Ejecución](#ejecución)
- [Gestión de base de datos](#gestión-de-base-de-datos)
- [Entorno de desarrollo](#entorno-de-desarrollo)
- [Despliegue](#despliegue)
- [Solución de problemas](#solución-de-problemas)
- [Ejecutar bot Webhook](#ejecutar-bot-webhook)

---

## Requisitos previos

### Requisitos del sistema

- **Sistema operativo**: Linux, macOS o Windows (requiere Docker CLI)
- **Memoria**: Mínimo 8GB, recomendado 16GB
- **Almacenamiento**: Mínimo 20GB de espacio disponible
- **CPU**: Recomendado 4 núcleos o más

> Nota (intención de diseño)
> El requisito central en Windows es que Docker CLI esté disponible, los comandos se pueden ejecutar directamente en PowerShell o Windows Terminal.
> Pero los contenedores aún necesitan un tiempo de ejecución Linux para alojarse:
> 1. La solución local suele ser Docker Desktop (generalmente dependiente del backend WSL2).
> 2. La alternativa es instalar solo Docker CLI localmente y reenviar a través de `docker context` a un host Docker Linux remoto.

### Requisitos de software

#### Software obligatorio

- **Docker o Podman** (entorno de tiempo de ejecución de contenedores)

```bash
docker --version
docker compose version
```

Instala usando el método oficial recomendado para tu plataforma:

- Linux: instala Docker Engine, Docker Desktop for Linux, o Podman del repositorio de tu distribución
- macOS: instala Docker Desktop o Podman Desktop
- Windows: instala Docker Desktop o Podman Desktop

**Nota importante**:

- Las dependencias de tiempo de ejecución como PostgreSQL ya están incluidas en el entorno contenedorizado
- Pero si necesitas ejecutar recetas `just` o scripts auxiliares del repositorio, el host aún requiere Python 3.8+
- No es necesario instalar PostgreSQL por separado en el host
- En Windows, los comandos se pueden ejecutar directamente en PowerShell o Windows Terminal, pero el despliegue aún requiere un tiempo de ejecución Linux Docker/Podman disponible. El despliegue local generalmente significa usar Docker Desktop con backend WSL2; también se puede usar Docker CLI/context local para reenviar a un host Docker Linux remoto.

- **Rust 1.85+** (solo necesario para construcción de desarrollo)

```bash
rustup update stable
```

Instala usando el método oficial rustup para tu plataforma:

- Linux/macOS: visita <https://rustup.rs>
- Windows: descarga y ejecuta `rustup-init.exe` desde <https://rustup.rs>, luego ejecuta `rustup update stable`

#### Software recomendado

- **just** (ejecutor de comandos)

```bash
  # usando cargo
  cargo install just

  # usando brew (macOS)
  brew install just
  ```

- **VS Code** con la extensión rust-analyzer instalada

---

## Instalación

### Paso 1: Clonar el repositorio

```bash
git clone https://github.com/celestia-island/entelecheia.git
cd entelecheia
```

### Paso 2: Configurar variables de entorno

```bash
# Edita la configuración después de crear .env desde .env.example
nano .env  # o usa tu editor favorito
```

Usa tu shell actual o administrador de archivos para copiar `.env.example` a `.env`.

Shell POSIX:

```bash
cp .env.example .env
```

PowerShell:

```powershell
Copy-Item .env.example .env
```

#### Configuración básica

```bash
# Configuración de base de datos (configurada automáticamente dentro del contenedor)
# DATABASE_URL=postgresql://entelecheia:password@localhost:5432/entelecheia
# DATABASE_MAX_CONNECTIONS=10

# Inicialización rápida de LLM, importar ApoRia después del inicio
# Proveedor único:
# LLM_API_KEY=tu-api-key-aqui
# LLM_BASE_URL=https://api.openai.com/v1
# LLM_MODEL=gpt-4
# Múltiples proveedores (separados por punto y coma):
# LLM_API_KEY=key1;key2
# LLM_BASE_URL=https://api.one/v1;https://api.two/v1
# LLM_PROTOCOL=openai;openai,api-key
# LLM_MODEL_DEEP=model-a1,model-a2;model-b1
# LLM_MODEL_NORMAL=model-a3;model-b2
# LLM_MODEL_BASIC=model-a4;model-b3

# Acceso directo por proveedor (recomendado)
OPENAI_API_KEY=tu-api-key-aqui
# ANTHROPIC_API_KEY=
# DEEPSEEK_API_KEY=
# DASHSCOPE_API_KEY=
# BIGMODEL_API_KEY=
# ZAI_API_KEY=

# Configuración de WebSocket
WS_BIND_ADDRESS=127.0.0.1:42470
WS_MAX_CONNECTIONS=100
```

#### Instrucciones de configuración de variables de entorno LLM

> **Aviso importante**: La configuración actual del proveedor LLM es gestionada de forma unificada por ApoRia. Las variables de entorno solo sirven como punto de entrada de arranque inicial, ya no son la fuente de configuración a largo plazo.

**Mecanismo de funcionamiento**:

1. Cuando TUI necesita iniciar automáticamente el servidor, lee las variables de inicialización rápida genéricas `LLM_*`, o variables a nivel de proveedor como `OPENAI_API_KEY`. La configuración de múltiples proveedores usa arrays paralelos separados por punto y coma: `LLM_API_KEY`, `LLM_BASE_URL`, `LLM_PROTOCOL`, `LLM_MODEL_DEEP`, `LLM_MODEL_NORMAL`, `LLM_MODEL_BASIC`. Las variables de entorno de paquetes de programación (como `BIGMODEL_API_KEY_CODING_PRO`) también admiten múltiples claves separadas por punto y coma, numeradas automáticamente `(#2)`, `(#3)`. Los proveedores personalizados muestran el nombre de dominio entre paréntesis.
1. Antes de que el servidor se inicie, TUI primero preescribe la configuración inicial del proveedor en `res/prompts/agents/aporia/config.toml`
1. Después de la preescritura, prevalece la configuración de ApoRia y la página Models de TUI
1. Los proveedores existentes con API key no vacía no serán sobrescritos por variables de entorno

**Uso recomendado**:

- Usa variables de entorno para completar el arranque inicial
- Posteriormente, mantén la configuración de forma unificada a través de la página Models o `res/prompts/agents/aporia/config.toml`

### Paso 3: Iniciar servicios

```bash
# Iniciar todos los servicios con Docker Compose
docker compose up -d

# O usar el comando just (si está instalado)
just dev
```

---

## Configuración

### Configuración del proveedor LLM

Entelecheia (玄枢) admite múltiples proveedores LLM. Configura tu proveedor preferido:

#### OpenAI

```bash
OPENAI_API_KEY=sk-...
```

#### Anthropic

```bash
ANTHROPIC_API_KEY=sk-ant-...
```

#### LLM local (Ollama)

```bash
# Configura el proveedor local a través de la página Models o res/prompts/agents/aporia/config.toml
# endpoint = http://localhost:11434
# model = llama2
```

### Configuración de Docker

```bash
# Docker socket (generalmente detectado automáticamente)
DOCKER_HOST=unix:///var/run/docker.sock

# Configuración del contenedor
CONTAINER_NETWORK=entelecheia-network
CONTAINER_REGISTRY=127.0.0.1:5000
```

---

## Construcción

### Construcción de desarrollo

```bash
# Construcción rápida para desarrollo
just build-dev
```

### Construcción de producción

```bash
# Construcción de lanzamiento optimizada
just build
```

### Construir componentes específicos

```bash
# Construir solo el servidor
cargo build -p scepter

# Construir solo TUI
cargo build -p entelecheia-tui

# Construir un agente específico
cargo build -p haplotes
```

### Artefactos de construcción

Una vez completada la construcción, encontrarás:

- **Binarios**: `target/debug/` o `target/release/`
- **Imágenes Docker**: construidas automáticamente durante `just dev`

---

## Ejecución

### Modo desarrollo

```bash
# Iniciar entorno de desarrollo completo (incluye TUI)
just dev

# Iniciar solo el servidor (sin TUI)
just dev --no-tui

# Inicio limpio (elimina todos los datos)
just dev-clean
```

### Modo producción

```bash
# Iniciar servidor
just server

# Iniciar cliente TUI
just tui

# Iniciar todos los agentes
just agents-up
```

### Parámetros de compatibilidad de terminal

TUI depende de secuencias de escape ANSI, eventos de ratón y renderizado de imágenes (protocolos Sixel/Kitty). En entornos de terminal restringidos — como sesiones SSH, consolas serie, ejecutores CI o emuladores de terminal antiguos — se pueden usar tres parámetros de degradación progresiva:

#### `--no-image-render`

Desactiva todo el renderizado de imágenes. El resto de funciones — color, ratón, actualización diferencial — permanecen completamente funcionales.

```bash
just tui -- --no-image-render
```

Escenario aplicable: terminales que admiten color y ratón pero carecen de soporte para protocolos de imagen Sixel/Kitty (el caso más común).

#### `--no-ansi`

Desactiva la captura de ratón y la escucha de teclas especiales. El color y la actualización diferencial (parcial) de pantalla se conservan. Útil cuando los eventos de ratón interfieren con la selección de terminal, copiar/pegar o el historial de desplazamiento.

```bash
just tui -- --no-ansi
```

Escenario aplicable: se necesita color pero la captura de ratón causa problemas (multiplexores de terminal, `screen`, configuración básica de `tmux`, etc.).

#### `--no-ansi-pure`

Modo monocromo puro — la degradación más agresiva. Desactiva todos los colores ANSI (forzando globalmente `Color::Reset`), desactiva la captura de ratón, realiza redibujado completo de pantalla por cada fotograma. El logo de inicio se reemplaza con una versión de arte ASCII puro. Este parámetro implica `--no-ansi`.

```bash
just tui -- --no-ansi-pure
```

Escenario aplicable: ejecución a través de SSH con soporte de terminal mínimo, consolas serie, `docker exec`, entornos CI, o cualquier terminal que no maneje correctamente los códigos de color ANSI.

#### Comparación de parámetros

| Función | Predeterminado | `--no-image-render` | `--no-ansi` | `--no-ansi-pure` |
| --- | --- | --- | --- | --- |
| Color | Completo | Completo | Completo | Desactivado |
| Captura de ratón | Sí | Sí | No | No |
| Renderizado de imágenes | Sí | No | No | No |
| Actualización de pantalla | Diferencial | Diferencial | Diferencial | Redibujado completo |
| Logo de inicio | ANSI color | ANSI color | ANSI color | ASCII puro |

### Gestión de servicios

```bash
# Verificar estado de servicios
just dev-status

# Ver registros
just dev-logs

# Detener servicios
just dev-down

# Forzar terminación de todos los servicios
just dev-kill
```

---

## Gestión de base de datos

### Inicializar base de datos

```bash
# Crear base de datos
just db-create

# Ejecutar migraciones
just db-migrate

# Inicializar con datos semilla
just db-init
```

### Operaciones de base de datos

```bash
# Verificar estado de la base de datos
just db-status

# Respaldar base de datos
just db-backup

# Restaurar base de datos
just db-restore backups/backup_xxx.sql

# Restablecer base de datos (advertencia: elimina todos los datos)
just db-reset
```

### Gestión de migraciones

```bash
# Crear nueva migración
cargo test -p scepter test_create_migration -- --nocapture --ignored

# Revertir última migración
just db-migrate-down
```

---

## Entorno de desarrollo

### Configuración del entorno

```bash
# Inicializar todas las dependencias
just init

# Verificar dependencias de Python

# Formatear código
just fmt

# Ejecutar verificación de código
just clippy
```

### Pruebas

```bash
# Ejecutar todas las pruebas
just test

# Ejecutar tipos específicos de pruebas
just test unit
just test integration
just test e2e
just test llm-providers

# Salida detallada
just test verbose
```

### Calidad de código

```bash
# Formatear código
just fmt

# Verificar formato
just fmt-check

# Ejecutar clippy
just clippy

# Verificación de tipos
just check
```

---

## Despliegue

### Despliegue Docker

#### Construir imagen

```bash
docker build -t entelecheia:latest .
```

#### Ejecutar contenedor

```bash
docker run -d --name entelecheia \
  --env-file .env \
  -p 8424:8424 \
  -v /var/run/docker.sock:/var/run/docker.sock \
  entelecheia:latest
```

### Despliegue con Docker Compose

```bash
# Iniciar todos los servicios
docker compose up -d

# Ver registros
docker compose logs -f

# Detener servicios
docker compose down
```

---

## Solución de problemas

### Problemas comunes

#### Permiso denegado de Docker

```bash
# Añadir usuario al grupo docker
sudo usermod -aG docker $USER

# Cerrar sesión y volver a iniciar
```

#### Puerto ya en uso

```bash
# Verificar el proceso que ocupa el puerto
lsof -i :8424

# Terminar el proceso
kill -9 <PID>
```

#### Fallo de construcción

```bash
# Limpiar artefactos de construcción
cargo clean

# Actualizar dependencias
cargo update

# Reconstruir
just build
```

#### El contenedor no puede iniciar

```bash
# Verificar registros de Docker
docker compose logs

# Reconstruir contenedor
docker compose down
docker compose build --no-cache
docker compose up -d
```

### Obtener ayuda

1. Busca en [GitHub Issues](https://github.com/celestia-island/entelecheia/issues)
1. Únete a nuestro [foro de discusión](https://github.com/celestia-island/entelecheia/discussions)

---

## Ejecutar bot Webhook

El bot Webhook se encuentra en `plugins/github-webhook/`. Cada plataforma tiene su propio directorio.

### Requisitos previos

- Python 3.10+ (bot actual)
- Node.js 18+ (futura migración a TypeScript)
- Token de bot para cada plataforma (consulta la [Guía de configuración de Webhook](webhook-setup.md))

### Ejecutar un solo bot

```bash
# GitHub
cd plugins/github-webhook/github
pip install -r requirements.txt
python bot.py

# Gitee
cd plugins/github-webhook/gitee
pip install -r requirements.txt
python bot.py

# Discord
cd plugins/github-webhook/discord
pip install -r requirements.txt
python bot.py
```

### Ejecutar todos los bots

```bash
just webhooks-up
```

### Variables de entorno

Copia el archivo de entorno de ejemplo y configúralo:

```bash
cp plugins/github-webhook/.env.example plugins/github-webhook/.env
```

Consulta la [Guía de configuración de Webhook](webhook-setup.md) para detalles de configuración específicos de cada plataforma.

---

## Siguientes pasos

- Lee la [Guía de fundamentos](fundamentals.md) para entender la arquitectura
- Explora la [documentación de agentes](../../agents/) para conocer los agentes disponibles

---

**¡Feliz construcción!** 🚀
