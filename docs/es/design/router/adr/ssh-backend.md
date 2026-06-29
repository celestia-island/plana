+++
title = "Backend SSH — russh con Conexión Compartida"
description = """Registro de decisión arquitectónica — Backend SSH — russh con Conexión Compartida."""
lang = "es"
category = "design"
subcategory = "router"
+++

# Backend SSH — russh con Conexión Compartida

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight necesita SSH para shell remota, operaciones de archivos y acceso a terminal. Inicialmente el código tenía tres implementaciones de manejador SSH separadas (FileHandler, SshHandler, TerminalHandler) con lógica de conexión duplicada. El proyecto exige Rust puro (sin ejecuciones externas `Command::new("ssh")`).

## Decisión

Usar `russh` (implementación SSH-2 en Rust puro) como backend SSH. Consolidar todas las implementaciones de manejador SSH en un único `DefaultSshHandler` en `remote/connection.rs` con una función compartida `connect_session()`. Todas las operaciones SSH (shell, archivos, terminal) comparten esta abstracción de conexión.

## Consecuencias

### Positivas

- Fuente única de verdad para la lógica de autenticación SSH
- Fácil de agregar un pool de conexiones posteriormente

### Negativas

- `russh` puede quedarse atrás respecto a OpenSSH en casos límite
- Sin reenvío de agente SSH incorporado aún
