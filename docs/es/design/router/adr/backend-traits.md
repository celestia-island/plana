+++
title = "Abstracciones de Traits TerminalBackend / ViewportBackend / FileBackend"
description = """Registro de decisión arquitectónica — Abstracciones de Traits TerminalBackend / ViewportBackend / FileBackend."""
lang = "es"
category = "design"
subcategory = "router"
+++

# Abstracciones de Traits TerminalBackend / ViewportBackend / FileBackend

- **Estado**: Aceptada
- **Fecha**: 2026-06-09
- **Autores**: Evernight Core Team

## Contexto

Evernight necesita interfaces de backend polimórficas para que los frontends (CLI, TUI, GUI) puedan consumir cualquier protocolo de manera uniforme. Sin traits compartidos, cada frontend necesitaría rutas de código específicas por protocolo para operaciones de terminal, visualización gráfica y archivos.

## Decisión

Definir tres traits asíncronos seguros para objetos en la raíz del crate (siempre disponibles, sin feature flag):

- **`TerminalBackend`** — `read` / `write` / `resize` / `close`
- **`ViewportBackend`** — `render` / `input` / `clipboard` / `close`
- **`FileBackend`** — `list` / `stat` / `get` / `put` / `rm` / `mkdir` / `rename`

Cada backend de protocolo implementa los traits relevantes. Los frontends consumen `Box<dyn TerminalBackend>`, etc.

## Consecuencias

### Positivas

- Los frontends son agnósticos al protocolo; nuevos backends (p. ej., RDP) se integran sin cambios en los frontends.

### Negativas

- Los objetos trait asíncronos requieren sobrecarga de `Box::pin`.
- El diseño de los traits debe ser estable, ya que cambiarlo rompe todos los implementadores.
