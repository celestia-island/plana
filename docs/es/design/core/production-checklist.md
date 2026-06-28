+++
title = "Lista de Verificación de Despliegue en Producción de Entelecheia"
description = """> Lista de verificación de 12 pasos para desplegar Entelecheia en producción."""
lang = "es"
category = "design"
subcategory = "core"
+++

# Lista de Verificación de Despliegue en Producción de Entelecheia

> Lista de verificación de 12 pasos para desplegar Entelecheia en producción.

## Pre-Despliegue

- [ ] **1. Elegir Modo de Base de Datos**
  - pglite embebido: binario único, sin BD externa. Adecuado para <50 agentes concurrentes.
  - PostgreSQL: recomendado para producción. Establecer `DATABASE_URL`.

  ```bash
  # Modo embebido
  docker run -d -p 8080:8080 -v data:/data entelecheia:latest

  # Modo PostgreSQL
  docker-compose up -d
  ```

- [ ] **2. Configurar Identidad de Usuario**

  ```bash
  export ENTELECHEIA_USER_UUID=$(uuidgen)
  ```

Este UUID es la identidad del propietario del espacio de trabajo. Todas las operaciones de agente están limitadas a él.

- [ ] **3. Configurar Proveedores LLM**

  ```bash
  entelecheia-cli config set-provider openai --api-key sk-...
  entelecheia-cli config set-provider anthropic --api-key sk-ant-...
  ```

Las claves API se encriptan en reposo con AES-256-GCM a través del agente Aporia.

- [ ] **4. Configurar Runtime de Contenedores**
  - Docker (predeterminado): `--container-backend docker`
  - Youki (OCI sin root): `--container-backend youki`
  - Verificar perfil seccomp: `configs/seccomp/`

- [ ] **5. Revisar Políticas de Seguridad**

  ```bash
  # Listar políticas de seguridad registradas
  entelecheia-cli security policy-list

  # Revisar configuración del centinela OreXis
  entelecheia-cli config show orexis
  ```

## Despliegue

- [ ] **6. Construir o Descargar Imagen**

  ```bash
  # Construir desde fuente
  docker build -t entelecheia:latest .

  # O usar versión de lanzamiento
  curl -fsSL https://raw.githubusercontent.com/celestia-island/entelecheia/main/scripts/deploy/install.sh | bash
  ```

- [ ] **7. Iniciar el Servicio**

  ```bash
  # Usando Docker Compose (recomendado)
  docker-compose up -d

  # O independiente
  docker run -d --name entelecheia \
    -p 8080:8080 \
    -v entelecheia-data:/data \
    -e ENTELECHEIA_USER_UUID=$ENTELECHEIA_USER_UUID \
    --restart unless-stopped \
    entelecheia:latest
  ```

- [ ] **8. Verificar Estado**

  ```bash
  entelecheia-cli status
  curl http://localhost:8080/health
  ```

- [ ] **9. Inicializar Imágenes Docker para Agentes**

  ```bash
  entelecheia-cli init-docker-images
  ```

Esto construye las imágenes de contenedor utilizadas por cada agente de Capa-1 para ejecución aislada.

## Post-Despliegue

- [ ] **10. Configurar Monitoreo**

  ```bash
  # Habilitar trazado
  export RUST_LOG=info,entelecheia=debug

  # Verificar línea de tiempo para problemas
  entelecheia-cli timeline list --agent orexis
  ```

- [ ] **11. Configurar Respaldos**
  - Modo embebido: respaldar directorio `/data`
  - PostgreSQL: `pg_dump` o archivado WAL
  - Registros de auditoría de línea de tiempo: exportar periódicamente

- [ ] **12. Prueba de Carga**

  ```bash
  # Enviar un mensaje de prueba
  entelecheia-cli send "Hola, verificar que el sistema está operativo"

  # Verificar estado del agente
  entelecheia-cli agent list

  # Verificar pista de auditoría
  entelecheia-cli trace-chain demiurge.001
  ```

## Endurecimiento de Seguridad (Recomendado)

| Verificación | Comando |
| --- | --- |
| Verificar que no hay secretos en env | `env \| grep -i key` |
| Revisar grupos RBAC | `entelecheia-cli security rbac-list` |
| Verificar límites de tasa | `entelecheia-cli config show channel.rate_limit` |
| Verificar aislamiento de contenedores | `docker inspect entelecheia \| grep SecurityOpt` |
| Revisar registro de auditoría OreXis | `entelecheia-cli logs --agent orexis --lines 100` |

## Solución de Problemas

| Síntoma | Diagnóstico |
| --- | --- |
| Agentes no responden | `entelecheia-cli status` → verificar que scepter está ejecutándose |
| Llamadas LLM fallan | Verificar claves API: `entelecheia-cli config show providers` |
| Errores de contenedor | `docker logs entelecheia` → buscar errores de Youki/Docker |
| Problemas de base de datos | Verificar `DATABASE_URL` o permisos del directorio de datos pglite |
| Permiso de herramienta denegado | `entelecheia-cli security policy-list` → revisar llamadas denegadas |
