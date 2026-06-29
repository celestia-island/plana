+++
title = "Analizador de Configuración SSH"
description = """Registro de decisión arquitectónica — Analizador de Configuración SSH."""
lang = "es"
category = "design"
subcategory = "router"
+++

# Analizador de Configuración SSH

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Los usuarios esperan que `evernight` lea `~/.ssh/config` para alias de host, jump hosts y archivos de clave, igual que el comando `ssh`. Sin esto, los usuarios deben repetir todos los parámetros de conexión en cada invocación.

## Decisión

Implementar un analizador de configuración SSH en Rust puro que maneje las directivas estándar: `Host`, `HostName`, `User`, `Port`, `IdentityFile`, `ProxyJump`, `ForwardAgent`, `ServerAliveInterval`, etc. Coincidencia de patrones glob para entradas `Host`. Integrado con el pool de conexiones para conexiones transparentes basadas en configuración.

## Consecuencias

### Positivas

- Compatibilidad directa con SSH; resolución de jump host desde la configuración.

### Negativas

- Se debe seguir la evolución de la sintaxis de configuración de OpenSSH.
- El soporte de la directiva `Match` es complejo.
