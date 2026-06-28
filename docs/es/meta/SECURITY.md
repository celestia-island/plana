+++
title = "Política de Seguridad"
description = """No abras issues públicos para vulnerabilidades de seguridad."""
lang = "es"
category = "meta"
+++

# Política de Seguridad

## Reportar una Vulnerabilidad

**No abras issues públicos para vulnerabilidades de seguridad.**

Repórtalas de forma privada a través de
[Avisos de Seguridad de GitHub](https://github.com/celestia-island/arona/security/advisories/new).
Si los Avisos de Seguridad de GitHub no están disponibles para ti, envía un correo al mantenedor a
security@celestia.world con una descripción clara y pasos de reproducción.

## Alcance

Dentro del alcance:

- Elusión de autenticación, debilidades JWT/OAuth, fallos en el manejo de sesiones
- Divulgación de claves API/credenciales o almacenamiento inadecuado
- Brechas en la aplicación de autorización y RBAC
- Vulnerabilidades de inyección (SQL, comandos, SSRF, XSS)
- Deserialización insegura, path traversal, SSRF
- Problemas que permitan escalada de privilegios o acceso entre tenants

Fuera del alcance:

- Vulnerabilidades en dependencias upstream no explotables a través de este proyecto
- Despliegues autoalojados con configuración insegura contraria a la guía documentada
- Denegación de servicio contra los endpoints públicos del proveedor LLM

## Respuesta

| Etapa | Objetivo |
| --- | --- |
| Acuse de recibo por agente | 10 minutos |
| Acuse de recibo humano | 1 día natural |
| Evaluación inicial | 3 días naturales |
| Corrección o mitigación | 30 días naturales (dependiendo de la severidad) |

Por favor, incluye: (1) el componente y versión afectados, (2) el vector de ataque
e impacto, (3) pasos de reproducción, y (4) mitigaciones sugeridas.

## Versiones Soportadas

Solo la última línea de lanzamiento en las ramas `main` / `dev` recibe correcciones
de seguridad.
