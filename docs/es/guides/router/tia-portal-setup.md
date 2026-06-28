+++
title = "Requisitos previos de TIA Portal — Conectar evernight"
description = """Cómo preparar una vez un PLC S7-1200/1500 en TIA Portal para que evernight pueda conectarse, autodescubrirse y leer/escribir el dispositivo sin más intervención humana."""
lang = "es"
category = "guides"
subcategory = "router"
+++

# Requisitos previos de TIA Portal — Conectar evernight

> **Objetivo**: preparar **una vez** su PLC Siemens S7-1200/1500 en TIA Portal
> para que evernight pueda conectarse, autodescubrirse y leer/escribir el
> dispositivo **sin más intervención humana**. Es una configuración única de
> propiedades de CPU — su lógica de programa ladder/SCL nunca se modifica.

evernight habla **dos canales** con un PLC Siemens. Elija según lo que exponga
su PLC:

| Canal | Puerto | Estilo de acceso | Preparación TIA | Recomendado para |
|-------|--------|------------------|-----------------|------------------|
| **S7comm** | 102 | R/W de bytes crudos de M / I / Q / DB | PUT/GET + DB no optimizados | legacy, ligero, sin licencia OPC UA |
| **OPC UA** | 4840 | simbólico, autodescriptivo | activar servidor integrado | **recomendado** — autodescubrimiento, DB optimizados OK |

Si puede activar OPC UA, preferirlo: evernight **recorre** automáticamente todo
el espacio de direcciones simbólico, sin entrada manual de símbolos.

---

## Ruta A — S7comm (acceso a registros crudos)

### A.1 Habilitar la comunicación PUT/GET

Los S7-1200/1500 bloquean por defecto la lectura/escritura S7 externa.

1. Abra el proyecto en **TIA Portal**.
2. En la vista de dispositivos/red, **haga clic en el CPU**.
3. Propiedades → **Protección y seguridad (Protection & Security) → Mecanismos de conexión (Connection mechanisms)**.
4. Marque **"Permitir acceso con comunicación PUT/GET desde interlocutor remoto (Permit access with PUT/GET communication from remote partner)"**.
5. Descargue la configuración de hardware al CPU.

### A.2 Hacer los DB objetivo no optimizados

El acceso optimizado a bloques (predeterminado en S7-1200/1500) no tiene offset
de byte fijo, por lo que la lectura por dirección absoluta falla. Para cada DB
que evernight deba leer/escribir:

1. Clic derecho en el DB → **Propiedades (Properties)**.
2. Desmarque **"Acceso optimizado a bloques (Optimized block access)"**.
3. Recompile y descargue.

> Los marcadores M y las imágenes de proceso I/Q siempre son direccionables por
> byte — sin cambios. Este paso solo afecta a los DB.

### A.3 Conectar desde evernight

```
s7://192.168.1.10:102?rack=0&slot=1
```

- S7-1200/1500: `rack=0, slot=1`
- S7-300: `slot=2`

Autodescubrimiento en código:

```rust
use evernight::protocol::auto_provision;

let profile = auto_provision("192.168.1.10").await?;
// profile.data_blocks / profile.db_structures ahora describen cada DB legible
```

---

## Ruta B — OPC UA (recomendado)

### B.1 Requisitos de firmware y licencia

- Firmware del CPU **V2.0+** (V2.5+ para métodos OPC UA).
- Una **licencia de ejecución SIMATIC OPC UA** acorde al CPU (asignada bajo
  Propiedades del CPU → **Licencias de ejecución (Runtime licenses) → OPC UA**).
  Requerida por cumplimiento.

### B.2 Activar el servidor OPC UA

1. **Haga clic en el CPU** en la vista de red/dispositivo.
2. Propiedades → **OPC UA → General**: introduzca un nombre de servidor.
3. Propiedades → **OPC UA → Servidor (Server)**: marque **"Activar servidor OPC UA (Activate OPC UA server)"**.
4. Asigne el servidor a la **interfaz PROFINET** que el cliente alcanzará.

### B.3 Exponer las variables simbólicas

- **OPC UA → Servidor → Interfaz de servidor (Server interface)**: seleccione
  **"Interfaz de servidor SIMATIC estándar (Standard SIMATIC server interface)"**
  para que cada variable/DB simbólico (incluidos los DB optimizados) se publique
  automáticamente.

### B.4 Autenticación y seguridad

- **OPC UA → Servidor → Autenticación de usuario (User authentication)**:
  anónima (para una LAN de confianza) o usuario/contraseña.
- **OPC UA → Servidor → Seguridad (Security)**: elija una política. `None` es
  lo más simple para una primera conexión; `Sign & Encrypt` para producción.
- En firmware **V3.1+ con TIA V19+**, conceda el rol funcional / derecho de
  ejecución **"acceso al servidor OPC UA (OPC UA server access)"** al usuario
  que se conecta.

### B.5 Confiar en el certificado del cliente

Los clientes OPC UA presentan un certificado X.509 al conectar; el PLC pone en
cuarentena los certificados desconocidos. Tras el primer intento de evernight:

1. TIA Portal → **CPU → Certificados (Certificates)** (online), **o**
2. **Servidor Web** del PLC → "Comunicación con clientes OPC UA", **o**
3. el gestor de certificados de la pantalla del CPU.

Luego **acepte/confíe** en el certificado cliente de evernight.

### B.6 (Opcional) Exportar el XML NodeSet de OPC UA

Un archivo NodeSet es un mapa sin conexión de todas las variables — útil para
planificar sin conexión en vivo:

1. Propiedades del CPU → **OPC UA → Servidor → Exportar (Export)**.
2. Haga clic en **"Exportar archivo XML de OPC UA (Export OPC UA XML file)"**,
   guarde el `*.Opc.Ua.NodeSet2.xml`.

### B.7 Descargar

Descargue la **configuración de hardware**. Son propiedades del CPU, no lógica
de programa — su código ladder/SCL queda intacto.

### B.8 Conectar desde evernight

URL del endpoint:

```
opc.tcp://192.168.1.10:4840
```

evernight se conecta como cliente OPC UA, **recorre** todo el árbol simbólico y
lee/escribe por nombre — sin entrada manual de símbolos, DB optimizados incluidos.

---

## Verificar la conectividad (sondeos sin riesgo)

Antes de accionar salidas, confirme que los canales están vivos con sondeos de
solo lectura:

```bash
# ¿Algo habla S7comm en el puerto 102?
evernight probe 192.168.1.10 --ports 102

# ¿Está arriba el servidor OPC UA en 4840?
evernight probe 192.168.1.10 --ports 4840
```

Ambos son handshakes pasivos — no leen ni escriben nada.

---

## Límites de seguridad

- **Nunca** ponga los enclavamientos de seguridad (paradas de emergencia, fines
  de carrera, sobrecarga) en la ruta S7/OPC UA. Manténgalos en el escaneo del
  PLC. Un enlace de red caído no debe desactivar una función de seguridad.
- El control evernight se adapta a cargas **lentas** (válvulas, máquinas de
  estados, conmutación de modo). La latencia de ida y vuelta S7/OPC UA es
  ~10–50 ms — suficiente para supervisión, demasiado lento para motion/servo.
- Prefiera escribir **bits de comando M** sobre los que actúa la lógica PLC
  existente (usted secuestra la fuente de disparo) en vez de escribir salidas Q
  directamente.

---

## Resolución de problemas

| Síntoma | Causa probable | Corrección |
|---------|---------------|------------|
| Conexión S7 rechazada / sin confirm COTP | PUT/GET no habilitado; rack/slot erróneos; cortafuegos | A.1; verifique `rack=0 slot=1` (1200/1500) |
| Lectura DB devuelve "optimized" / InvalidAddress | Acceso optimizado a bloques activo | A.2 — desmarcar acceso optimizado, recompilar |
| Endpoint OPC UA inalcanzable | Servidor no activado; no descargado; licencia faltante | B.2 / B.7 / B.1 |
| OPC UA conecta y luego rechaza | Certificado de cliente no confiable | B.5 |
| Recorrido devuelve vacío | Interfaz SIMATIC estándar no habilitada | B.3 |

---

## Referencias

- [Activación del servidor OPC UA (S7-1500) — docs STEP 7 V20](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/enabling-the-opc-ua-server-s7-1500-s7-1500t)
- [Acceso al servidor OPC UA (URL del endpoint / puerto 4840)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/access-to-the-opc-ua-server-s7-1500-s7-1500t)
- [Exportar archivo XML de OPC UA (NodeSet)](https://docs.tia.siemens.cloud/r/en-us/v20/configuring-automation-systems/using-opc-ua-communication-s7-1200-s7-1500-s7-1500t/using-the-s7-1500-as-an-opc-ua-server-s7-1500-s7-1500t/configuring-the-opc-ua-server-s7-1500-s7-1500t/accessing-opc-ua-server-data-s7-1500-s7-1500t/export-opc-ua-xml-file-s7-1500-s7-1500t)
