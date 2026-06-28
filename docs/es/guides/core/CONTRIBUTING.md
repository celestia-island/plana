+++
title = "Guía de contribución (Entelecheia)"
description = """> Este documento es la versión en español de la política de contribución. Para comandos de construcción e instrucciones detalladas de instalación, consulta el documento en inglés en la raíz del repositorio"""
lang = "es"
category = "guides"
subcategory = "core"
+++

# Guía de contribución (Entelecheia)

> Este documento es la versión en español de la política de contribución. Para comandos de construcción e instrucciones detalladas de instalación, consulta el documento en inglés en la raíz del repositorio
> [`CONTRIBUTING.md`](../../../CONTRIBUTING.md); los comandos en sí no se traducen. En caso de ambigüedad, prevalece la versión en inglés.

## Política de contribución (lee esto primero)

Entelecheia puede impulsar sistemas físicos e industriales, por lo tanto, **la estabilidad y seguridad tienen prioridad sobre el volumen de contribuciones**. Antes de enviar
un Pull Request, por favor lee esta sección.

- **El umbral de fusión es alto, no es una hoja de ruta pública.** Enviar un PR no significa que será fusionado. Solo aceptamos cambios deliberadamente escasos,

que se ajusten a la arquitectura y pasen la revisión. Esto es intencional, no es descortesía.

- **Contribuciones bienvenidas:** reportes de bugs, correcciones enfocadas, mejoras de alcance claro en la **periferia** (plugins Layer 3, perfiles de dispositivo,

adaptadores de proveedor LLM, integraciones, documentación), y discusiones de diseño antes de escribir código.

- **Generalmente no se fusionan:** reescrituras masivas no propuestas, cambios arquitectónicos sin discusión de diseño previa, PR masivos "vibe-coded",

cualquier cambio que reduzca el umbral de seguridad o corrección del núcleo, y modificaciones al núcleo crítico de seguridad sin invitación y revisión extendida.

- **Núcleo vs. Periferia.** El núcleo (orquestación, micronúcleo, seguridad) mantiene los estándares más estrictos y es mantenido principalmente por el equipo central.

La periferia es donde las contribuciones externas son más útiles y tienen más probabilidades de ser aceptadas.

- **Debes firmar el CLA.** Cada contribución aceptada requiere la firma del Acuerdo de Licencia de Contribuidor, ver [`CLA.md`](../../../CLA.md).

Los envíos deben incluir `Signed-off-by` (`git commit -s`).

> **La licencia se abrirá, el umbral de fusión no.** El **2030-01-01**, este proyecto pasará de BUSL-1.1 a SySL-1.0 (a elección del receptor), ver [`LICENSE`](../../../LICENSE). Esto flexibiliza *lo que puedes hacer con el código*, pero
> **no** reduce el umbral de revisión, no cancela el CLA, ni significa que aceptaremos más PR. La política de contribución permanece sin cambios antes y después de la fecha de cambio.

## Seguridad

**No** reportes vulnerabilidades de seguridad mediante issues públicos. Por favor, repórtalas de forma privada a través de
[GitHub Security Advisories](https://github.com/celestia-island/entelecheia/security/advisories/new).
El modelo de amenazas y el SLA de respuesta se encuentran en [`SECURITY.md`](../../../SECURITY.md).

## Código de conducta

Por favor, mantén el respeto, la constructividad y la inclusión. Seguimos el [Contributor Covenant Code of Conduct](../../../CODE_OF_CONDUCT.md).

## Proceso de Pull Request

1. Haz un Fork y crea una rama desde `main`.
1. Para cambios grandes, discute primero en un issue.
1. Haz commits atómicos, siguiendo Conventional Commits.
1. Asegúrate de que `just ci` (o el comando CI del repositorio) pase.
1. Firma el CLA y añade `Signed-off-by`.
1. Responde a los comentarios de revisión; usa force-push solo para rebase.

## Licencia y CLA

Se adopta **BUSL-1.1**, con **Fecha de Cambio el 2030-01-01**, momento en el cual pasará a **SySL-1.0** a elección del receptor. Actualmente, para operaciones internas, académicas, gubernamentales, educativas y usos no comerciales, ya equivale a SySL-1.0 (ver
la Concesión de Uso Adicional en [`LICENSE`](../../../LICENSE)). Los usos comerciales restringidos (alojamiento, reventa, venta como servicio con cambio de marca) requieren una licencia comercial separada antes de la Fecha de Cambio.

Al enviar una contribución, aceptas que esta se licencie bajo la licencia del proyecto y firmas el CLA ([`CLA.md`](../../../CLA.md)). El CLA
otorga al proyecto una licencia permisiva **con derechos de sublicencia incluidos**, permitiendo que el proyecto mantenga la ruta BUSL→SySL y ajuste la licencia en el futuro.
