+++
title = "Esquema URI de Entradas de Conexión"
description = """Registro de decisión arquitectónica — Esquema URI de Entradas de Conexión."""
lang = "es"
category = "design"
subcategory = "router"
+++

# Esquema URI de Entradas de Conexión

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

El catálogo de conexiones necesita una forma uniforme de representar diferentes conexiones de protocolo (SSH, VNC, RDP, serial, Docker). Un esquema URI proporciona descriptores legibles y serializables.

## Decisión

Usar esquemas URI específicos por protocolo:

- `ssh://usuario@host:puerto`
- `vnc://host:5900`
- `rdp://host:3389`
- `serial:///dev/ttyUSB0?baud=9600`
- `docker:///var/run/docker.sock?container=nombre`

`ConnectionEntry` analiza las URIs en estructuras tipadas con esquema, host, puerto, nombre de usuario, ruta y parámetros de consulta. El catálogo es un árbol de nodos `ConnectionCategory` que contienen entradas.

## Consecuencias

### Positivas

- Formato URI familiar; fácilmente serializable; permite compartir conexiones mediante copiar y pegar.

### Negativas

- Algunos protocolos no se adaptan limpiamente a URIs (p. ej., contexto de Kubernetes).
- Los parámetros de cadena de consulta no están estructurados.
