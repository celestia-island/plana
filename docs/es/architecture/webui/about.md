+++
title = "Acerca de Shittim Chest"
description = """Versión 0.1.0"""
lang = "es"
category = "architecture"
subcategory = "webui"
+++

# Shittim Chest (什亭之匣)

**Versión 0.1.0**

Shittim Chest es la interfaz de usuario para la plataforma de colaboración multi-agente [entelecheia](https://github.com/celestia-island/entelecheia), construida con Rust y Vue 3.

## Arquitectura

Shittim Chest consta de varios componentes que trabajan juntos para proporcionar una experiencia de usuario completa:

- **arona** — La interfaz de chat que estás usando actualmente, con respuestas en streaming, generación de imágenes, monitorización de estado de agentes, ventana de pensamiento, visor de dispositivos remotos y soporte multi-idioma.
- **`shittim_chest`** — El backend unificado en Rust + Axum que maneja autenticación (JWT + OAuth), enrutamiento LLM independiente, API de chat, generación de imágenes, ingreso de webhooks, proxy scepter y señalización de dispositivos remotos.

## Relación con Entelecheia

[entelecheia](https://github.com/celestia-island/entelecheia) es el motor central de orquestación multi-agente. Proporciona el runtime de agentes (scepter, 13 agentes especializados, runtime Cosmos/IEPL). Shittim Chest maneja todo con lo que el usuario interactúa directamente — identidad, presentación y comunicación.

Los dos proyectos están separados por diseño: entelecheia gestiona la orquestación de agentes, mientras que shittim-chest gestiona la identidad de usuario y la presentación. Se comunican mediante HTTP/WebSocket autenticado con JWT. Las credenciales de inicio de sesión residen en `shittim_chest_db`; los permisos y datos de identidad residen en entelecheia_db. Esta separación permite que la interfaz de usuario evolucione independientemente del núcleo de agentes.

## Relación con Hikari

[hikari](https://github.com/celestia-island/hikari) es la capa de puerta de enlace y enrutamiento para el ecosistema Celestia Island. Sirve como punto de entrada para todo el tráfico externo, manejando el enrutamiento de solicitudes, balanceo de carga y funcionalidad de API gateway entre shittim-chest, entelecheia y otros servicios.

## Relación con Tairitsu

[tairitsu](https://github.com/celestia-island/tairitsu) es el framework de aplicaciones nativas multiplataforma para el ecosistema Celestia Island. Proporciona clientes de escritorio y móviles basados en Tauri que envuelven arona como una aplicación nativa, junto con la infraestructura de automatización de navegadores y pruebas que impulsa el flujo de trabajo de desarrollo.

## Licencia

Shittim Chest está licenciado bajo la **Business Source License 1.1 (BSL-1.1)**.

Para **uso no comercial** — incluyendo operaciones internas, investigación académica, enseñanza, estudio personal, evaluación, uso gubernamental y de servicio público, y uso educativo — los derechos concedidos son equivalentes a la **Synthetic Source License 1.0 (SySL-1.0)** (la "Licencia de Uso Libre"). Puedes usar, estudiar, modificar y ejecutar libremente el software para estos fines.

El **uso comercial** — como ofrecer el software como servicio alojado a terceros, redistribuirlo como producto independiente, o usarlo como componente central de una oferta comercial — requiere una licencia comercial separada del Licenciante.

Consulta el [texto completo de la licencia](https://github.com/celestia-island/shittim-chest/blob/main/LICENSE) para más detalles.

---

Construido con ❤ por [Celestia Island](https://github.com/celestia-island).
