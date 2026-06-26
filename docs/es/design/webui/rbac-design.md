+++
title = "Documento de Diseño Detallado del Sistema RBAC"
description = """Implementar un sistema completo de control de acceso basado en roles para Shittim Chest, con soporte para:"""
lang = "es"
category = "design"
subcategory = "webui"
+++

# Documento de Diseño Detallado del Sistema RBAC

## 1. Objetivo

Implementar un sistema completo de control de acceso basado en roles para Shittim Chest, con soporte para:

- **Gestión de usuarios**: Los administradores pueden invitar/crear/deshabilitar/eliminar usuarios
- **Gestión de grupos**: Soporte para grupos de cuentas, los usuarios pueden pertenecer a múltiples grupos
- **Permisos de grano fino**: Controlar si los usuarios pueden añadir/modificar/usar proveedores de modelos específicos, herramientas MCP, Agentes Layer3, canales IM, etc.
- **Interruptores de funciones**: Controlar si los usuarios pueden usar funciones avanzadas como el modo de crucero automático
- **Modos de autorización flexibles**: Los administradores pueden elegir configuración global unificada, configuración individual por cuenta o compartida por grupo de cuentas

## 2. Conceptos Centrales

### 2.1 Roles (Role)

| Rol | Descripción |
|------|------|
| `admin` | Superadministrador, posee todos los permisos, puede gestionar RBAC |
| `operator` | Personal de operaciones, puede gestionar la mayoría de los recursos (proveedores, canales, Agentes, etc.) |
| `member` | Miembro ordinario, puede usar los recursos autorizados |
| `viewer` | Usuario de solo lectura, solo puede ver, no modificar |

Los roles son **predefinidos**, no se proporcionan roles personalizados (implementación simplificada). Cada usuario puede tener un rol principal.

### 2.2 Permisos (Permission)

Formato de permiso: `<resource>.<action>`

| Categoría | Permiso | Descripción |
|------|------|------|
| **Proveedor** | `provider.list` | Ver lista de proveedores |
| | `provider.create` | Añadir proveedor |
| | `provider.update` | Modificar configuración de proveedor |
| | `provider.delete` | Eliminar proveedor |
| | `provider.use` | Usar el modelo del proveedor para conversación |
| **Herramientas MCP** | `mcp.list` | Ver lista de herramientas MCP |
| | `mcp.create` | Registrar herramienta MCP |
| | `mcp.update` | Modificar configuración de herramienta MCP |
| | `mcp.delete` | Eliminar herramienta MCP |
| | `mcp.use` | Usar herramienta MCP en conversación |
| **Agente** | `agent.list` | Ver lista de Agentes |
| | `agent.create` | Crear Agente |
| | `agent.update` | Modificar configuración de Agente |
| | `agent.delete` | Eliminar Agente |
| | `agent.use` | Usar Agente en modo análisis |
| **Canal IM** | `channel.list` | Ver lista de canales IM |
| | `channel.create` | Crear canal IM |
| | `channel.update` | Modificar configuración de canal |
| | `channel.delete` | Eliminar canal |
| | `channel.use` | Enviar/recibir mensajes a través del canal |
| **Modo Crucero** | `yolo.use` | Usar modo de crucero automático |
| **Espacio de trabajo** | `workspace.list` | Ver espacios de trabajo |
| | `workspace.create` | Crear espacio de trabajo |
| | `workspace.manage` | Gestionar espacio de trabajo (eliminar, exportar) |
| **Dispositivo** | `device.list` | Ver dispositivos remotos |
| | `device.connect` | Conectar a dispositivo remoto |
| **Sistema** | `system.read` | Ver configuración del sistema |
| | `system.write` | Modificar configuración del sistema |
| | `rbac.manage` | Gestionar RBAC (usuarios/grupos/permisos) |
| **OAuth** | `oauth.read` | Ver configuración OAuth |
| | `oauth.write` | Modificar configuración OAuth |

### 2.3 Permisos Predeterminados por Rol

| Permiso | admin | operator | member | viewer |
|------|-------|----------|--------|--------|
| `provider.*` | ✅ | ✅ | `list` + `use` | `list` |
| `mcp.*` | ✅ | ✅ | `list` + `use` | `list` |
| `agent.*` | ✅ | ✅ | `list` + `use` | `list` |
| `channel.*` | ✅ | ✅ | `list` + `use` | `list` |
| `yolo.use` | ✅ | ✅ | ❌ (desactivado por defecto) | ❌ |
| `workspace.*` | ✅ | ✅ | `list` + `create` | `list` |
| `device.*` | ✅ | ✅ | `list` + `connect` | `list` |
| `system.*` | ✅ | ❌ | ❌ | ❌ |
| `rbac.manage` | ✅ | ❌ | ❌ | ❌ |
| `oauth.*` | ✅ | ✅ | ❌ | ❌ |

### 2.4 Modos de Autorización

Para recursos como proveedores, MCP, Agentes, canales, se soportan tres modos de autorización:

| Modo | Descripción | Escenario aplicable |
|------|------|---------|
| **Configuración global** | Todos los usuarios comparten los mismos permisos | Equipos pequeños, uso personal |
| **Por usuario** | Cada usuario tiene permisos de recursos independientes | Escenarios que requieren control fino |
| **Por grupo** | Usuarios del mismo grupo comparten permisos | División por departamento/equipo |

El administrador selecciona el modo de autorización en la página «Matriz de permisos» y luego configura reglas específicas de permitir/denegar.

**Prioridad**: Por usuario > Por grupo > Configuración global > Permisos predeterminados del rol

## 3. Esquema de Base de Datos

### 3.1 Nuevas Tablas

#### `rbac_groups` — Grupos de usuarios

```sql
CREATE TABLE rbac_groups (
    id          UUID PRIMARY KEY,
    name        VARCHAR(64) NOT NULL UNIQUE,
    description TEXT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

#### `rbac_user_groups` — Asociación usuario-grupo

```sql
CREATE TABLE rbac_user_groups (
    id         UUID PRIMARY KEY,
    user_id    UUID NOT NULL REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id   UUID NOT NULL REFERENCES rbac_groups(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, group_id)
);
```

#### `rbac_grants` — Concesiones de permisos (tabla unificada)

```sql
CREATE TABLE rbac_grants (
    id           UUID PRIMARY KEY,
    -- Objetivo de la concesión (elegir uno)
    scope        VARCHAR(16) NOT NULL, -- 'global' | 'group' | 'user'
    user_id      UUID REFERENCES auth_users(id) ON DELETE CASCADE,
    group_id     UUID REFERENCES rbac_groups(id) ON DELETE CASCADE,
    -- Permiso
    permission   VARCHAR(64) NOT NULL, -- ej. 'provider.use', 'yolo.use'
    resource_id  VARCHAR(128),         -- opcional: limitar a un recurso específico (nombre de proveedor, id de canal, etc.), NULL significa todos los recursos de esa categoría
    -- Tipo de concesión
    granted      BOOLEAN NOT NULL DEFAULT TRUE, -- TRUE=permitir, FALSE=denegar
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Restricción: scope y la FK correspondiente deben coincidir
    CONSTRAINT rbac_grants_scope_check CHECK (
        (scope = 'global' AND user_id IS NULL AND group_id IS NULL) OR
        (scope = 'user'   AND user_id IS NOT NULL AND group_id IS NULL) OR
        (scope = 'group'  AND user_id IS NULL AND group_id IS NOT NULL)
    )
);
CREATE INDEX idx_rbac_grants_user ON rbac_grants(user_id);
CREATE INDEX idx_rbac_grants_group ON rbac_grants(group_id);
CREATE INDEX idx_rbac_grants_permission ON rbac_grants(permission);
```

### 3.2 Modificar Tablas Existentes

#### `auth_users` añadir campo de rol

```sql
ALTER TABLE auth_users ADD COLUMN role VARCHAR(16) NOT NULL DEFAULT 'member';
-- Migración: usuarios con is_admin=true se establecen como 'admin'
UPDATE auth_users SET role = 'admin' WHERE is_admin = TRUE;
```

Mantener el campo `is_admin` para compatibilidad, pero el nuevo código prioriza `role`.

### 3.3 Lógica de Verificación de Permisos (pseudocódigo)

```rust
fn has_permission(user, permission, resource_id=None) -> bool {
    // 1. El rol admin pasa directamente
    if user.role == "admin" { return true; }

    // 2. Recopilar todas las concesiones coincidentes, ordenadas por prioridad
    let grants = [];

    // 2a. Permisos predeterminados del rol (prioridad más baja)
    grants.push(role_defaults(user.role, permission));

    // 2b. Configuración global
    grants.extend(query_grants(scope="global", permission, resource_id));

    // 2c. Configuración de grupo (todos los grupos a los que pertenece el usuario)
    for group in user.groups:
        grants.extend(query_grants(scope="group", group_id=group.id, permission, resource_id));

    // 2d. Configuración a nivel de usuario (prioridad más alta)
    grants.extend(query_grants(scope="user", user_id=user.id, permission, resource_id));

    // 3. Por prioridad: user > group > global > role_default
    // Dentro del mismo scope, denied prevalece sobre granted
    // Cualquier denied en scope user → rechazar
    // Cualquier denied en scope group → rechazar (a menos que scope user granted)
    // Resultado final
    resolve_grants(grants)
}
```

## 4. Diseño de API

### 4.1 Gestión de Usuarios (`/api/rbac/users`)

| Método | Ruta | Permiso | Descripción |
|------|------|------|------|
| GET | `/api/rbac/users` | `rbac.manage` | Listar todos los usuarios (incluyendo rol, grupos) |
| POST | `/api/rbac/users` | `rbac.manage` | Invitar usuario (enviar correo o crear cuenta) |
| PUT | `/api/rbac/users/:id` | `rbac.manage` | Actualizar rol del usuario, habilitar/deshabilitar |
| DELETE | `/api/rbac/users/:id` | `rbac.manage` | Eliminar usuario |

### 4.2 Gestión de Grupos (`/api/rbac/groups`)

| Método | Ruta | Permiso | Descripción |
|------|------|------|------|
| GET | `/api/rbac/groups` | `rbac.manage` | Listar todos los grupos |
| POST | `/api/rbac/groups` | `rbac.manage` | Crear grupo |
| PUT | `/api/rbac/groups/:id` | `rbac.manage` | Actualizar grupo (nombre, descripción) |
| DELETE | `/api/rbac/groups/:id` | `rbac.manage` | Eliminar grupo |
| POST | `/api/rbac/groups/:id/members` | `rbac.manage` | Añadir miembro |
| DELETE | `/api/rbac/groups/:id/members/:userId` | `rbac.manage` | Eliminar miembro |

### 4.3 Gestión de Permisos (`/api/rbac/grants`)

| Método | Ruta | Permiso | Descripción |
|------|------|------|------|
| GET | `/api/rbac/grants` | `rbac.manage` | Listar todas las reglas de permiso (soporta filtros ?scope=&permission=) |
| PUT | `/api/rbac/grants` | `rbac.manage` | Establecer permisos por lotes (pasar lista completa de reglas, sobrescribe las reglas del scope correspondiente) |
| DELETE | `/api/rbac/grants/:id` | `rbac.manage` | Eliminar una regla individual |

### 4.4 Verificación de Permisos (`/api/rbac/check`)

| Método | Ruta | Permiso | Descripción |
|------|------|------|------|
| GET | `/api/rbac/check?permission=xxx&resource_id=yyy` | (cualquier usuario autenticado) | Verificar si el usuario actual tiene el permiso especificado |
| GET | `/api/rbac/my-permissions` | (cualquier usuario autenticado) | Devolver la lista de todos los permisos efectivos del usuario actual |

### 4.5 Modificación de Visibilidad de Recursos

Las APIs de recursos existentes necesitan añadir filtrado de permisos:

- `GET /api/chat/providers` → solo devolver proveedores para los que el usuario actual tenga permiso `provider.list`, y solo mostrar modelos con permiso `provider.use`
- `GET /api/channel` → solo devolver canales con permiso `channel.list`
- Antes de iniciar el modo crucero → verificar permiso `yolo.use`

## 5. Diseño del Frontend (Plana)

### 5.1 Refactorización de RbacView

Dividido en tres pestañas:

#### Pestaña 1: Gestión de Usuarios
- Tabla de lista de usuarios: avatar, nombre de usuario, correo electrónico, rol (selector desplegable), etiquetas de grupo, estado (activo/deshabilitado), operaciones
- Botón de invitar usuario → abre Modal (introducir nombre de usuario/correo/contraseña, seleccionar rol)
- Operaciones de fila: editar rol, deshabilitar/habilitar, eliminar

#### Pestaña 2: Gestión de Grupos
- Tabla de lista de grupos: nombre, descripción, número de miembros, operaciones
- Crear grupo → abre Modal
- Hacer clic en grupo → expandir lista de miembros, se puede añadir/eliminar miembros

#### Pestaña 3: Matriz de Permisos
- Esquina superior izquierda seleccionar modo de autorización: Global / Por grupo / Por usuario
- Después de seleccionar grupo o usuario, mostrar tabla de matriz de permisos:
  - Filas: categorías de recursos (Proveedor, MCP, Agente, Canal, Modo Crucero...)
  - Columnas: operaciones (listar, crear, modificar, eliminar, usar)
  - Celdas: cambio de tres estados (✅ Permitir / ❌ Denegar / ➖ Heredar predeterminado)
- Control fino de ID de recurso específico (ej. solo permitir usar un proveedor concreto)

### 5.2 Control de Permisos de Navegación

- Los elementos de la barra lateral se muestran/ocultan dinámicamente según los permisos del usuario actual
- Los guards de ruta añaden verificación de permisos, redirigiendo a la página 403 sin permiso
- Los botones de operación (como "Añadir proveedor") se muestran/ocultan según los permisos

## 6. Pasos de Implementación

### Fase 1: Backend Básico
1. Nueva migración de base de datos (tablas rbac_groups, rbac_user_groups, rbac_grants + campo auth_users.role)
2. Nuevos modelos de entidad SeaORM
3. Implementar rutas API RBAC (CRUD de users, groups, grants)
4. Implementar middleware/extractor de verificación de permisos
5. Añadir campo role en los claims JWT

### Fase 2: Integración del Backend
6. Añadir verificación de permisos en las APIs de recursos existentes (providers, channels, etc.)
7. Implementar `/api/rbac/check` y `/api/rbac/my-permissions`
8. Modificar las solicitudes de recursos de arona para adaptarse al filtrado de permisos

### Fase 3: UI del Frontend
9. Refactorizar RbacView de arona (tres pestañas: Usuarios/Grupos/Matriz de permisos)
10. Implementar guards de permisos en la barra lateral y las rutas
11. El lado de arona oculta/deshabilita funciones según los permisos (como el botón de modo crucero)

## 7. Consideraciones de Seguridad

- Los permisos del rol `admin` no pueden ser sobrescritos por rbac_grants (pase directo hardcodeado)
- La verificación de permisos se ejecuta de manera unificada en la capa de middleware, sin depender de verificaciones manuales en el código de negocio
- Las operaciones sensibles (eliminar usuario, modificar permisos) registran logs de auditoría
- El JWT solo contiene el role, los permisos específicos se consultan en tiempo real desde la BD cada vez (evitando que los tokens no se actualicen tras cambios de permisos)
