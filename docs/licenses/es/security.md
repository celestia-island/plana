> **Nota**: Esta es una traducción de referencia comunitaria. En caso de discrepancia, prevalece la versión en inglés `SECURITY.md` en la raíz del repositorio.

# Política de seguridad

## Reportar una vulnerabilidad

**No abra issues públicos para vulnerabilidades de seguridad.**

Repórtelas de forma privada a través de
[GitHub Security Advisories](https://github.com/celestia-island/arona/security/advisories/new).
Si no tiene acceso a GitHub Security Advisories, envíe un correo electrónico al responsable a
security@celestia.world con una descripción clara y los pasos de reproducción.

## Alcance

Dentro del alcance:

- Omisión de autenticación, debilidades de JWT/OAuth, fallos de manejo de sesión
- Divulgación de clave de API / credenciales o almacenamiento inadecuado
- Brechas de autorización y aplicación de RBAC
- Vulnerabilidades de inyección (SQL, comando, SSRF, XSS)
- Deserialización insegura, path traversal, SSRF
- Problemas que permiten escalada de privilegios o acceso entre inquilinos

Fuera del alcance:

- Vulnerabilidades en dependencias upstream que no sean explotables a través de este proyecto
- Despliegues autoalojados con configuración insegura contraria a la guía documentada
- Ataques de denegación de servicio contra los endpoints del proveedor LLM público

## Respuesta

| Etapa | Objetivo |
| --- | --- |
| Acuse de recibo del Agent | 10 minutos |
| Acuse de recibo humano | 1 día natural |
| Evaluación inicial | 3 días naturales |
| Corrección o mitigación | 30 días naturales (depende de la gravedad) |

Incluya por favor: (1) el componente y la versión afectados, (2) el vector de ataque y el impacto, (3) los pasos de reproducción y (4) las mitigaciones sugeridas.

## Versiones compatibles

Solo la última línea de versión en las ramas `main` / `dev` recibe correcciones de seguridad.
