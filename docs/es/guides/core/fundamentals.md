+++
title = "Conceptos fundamentales"
description = """> Explicación de conceptos basada en la realidad actual del código"""
lang = "es"
category = "guides"
subcategory = "core"
+++

# Conceptos fundamentales

> Explicación de conceptos basada en la realidad actual del código

## Descripción general

Entelecheia (玄枢) es una plataforma multi-agente que utiliza una superficie de herramientas reducida visible para el modelo, un tiempo de ejecución compartido y múltiples puntos de entrada de cliente. Dado que el repositorio contiene simultáneamente implementaciones actuales, capacidades experimentales y documentos de diseño, esta guía solo explica los conceptos centrales que ya están activos en el código actual.

## Conceptos centrales

### Agent

Un Agent es un rol de tiempo de ejecución con prompt, skill y herramientas MCP.

- Layer1 es la capacidad central actual de la plataforma.
- El Layer2 integrado activo en el workspace actual es Web Automation.
- Layer3 (fase de diseño) está planificado para cargarse desde el directorio `.amphoreus/` — aún no implementado.

### Superficie de herramientas Exec-Only

El modelo no ve directamente todas las herramientas MCP. Las principales herramientas visibles para el modelo actualmente son:

- `exec`
- `write_to_var`
- `write_to_var_json`

Internamente en tiempo de ejecución, el código en `exec` puede invocar funciones de herramienta mediante importación de módulos ES (por ejemplo, `import { tool } from 'agent'`).

### Herramientas MCP

Las herramientas MCP son interfaces de capacidad estructuradas internas.

- Algunas ya están realmente implementadas.
- Algunas son implementaciones parciales.
- Otras siguen siendo stubs o esqueletos de validación de parámetros.

Por lo tanto, no se debe asumir por defecto que cada herramienta mencionada en la documentación ya está disponible de forma estable.

### Skill

Una Skill es un flujo de trabajo definido mediante prompt, que hace referencia a herramientas relevantes y, a veces, a otras skills.

- Algunas skills ya pueden impulsar flujos de trabajo reales.
- Otras skills están más cerca de ser documentos SOP que cadenas de automatización completas.

### Niveles

| Nivel | Significado actual |
| --- | --- |
| Layer1 | Agentes centrales compilados y habilitados en el workspace |
| Layer2 | Web Automation como agente de dominio integrado activo, más algunos diseños archivados |
| Layer3 | Agent personalizado por el usuario (en planificación, aún no implementado) |

## Clientes

### TUI

El punto de entrada de usuario más completo y maduro actualmente es TUI.

### WebUI

La Web UI (chat arona) y el panel de administración (plana) se han migrado al repositorio hermano [shittim-chest](https://github.com/celestia-island/shittim-chest) y se han eliminado de este repositorio; la interfaz preferida de este repositorio es TUI.

### CLI

CLI ya existe, pero algunos comandos aún muestran salidas de marcador de posición.

### Cliente Tauri

El código de escritorio y móvil ya existe en el repositorio hermano [shittim-chest](https://github.com/celestia-island/shittim-chest), pero es más apropiado considerarlo como integración temprana. La integración IDE (VS Code, IntelliJ) también se encuentra en shittim-chest.

## Expresión conservadora del modelo de seguridad

- Ya existen capacidades de autenticación JWT y API key.
- Ya existe mapeo RBAC para rutas HTTP, WebSocket y MCP conocidas.
- Ya existe capacidad de almacenamiento cifrado de claves de proveedor.
- El endurecimiento de contenedores y la integridad de auditoría aún son incompletos.

A menos que se haya verificado la ruta de código específica, no se debe asumir que TLS bidireccional, tokens de capacidad completos o aplicación estricta de políticas en toda la cadena sean hechos actuales.
